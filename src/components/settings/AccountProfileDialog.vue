<script setup lang="ts">
import { Input, Snackbar } from "@varlet/ui";
import { computed, nextTick, ref, useId, watch } from "vue";
import type { LightNovelUser } from "../../domain/auth";
import { setAvatar } from "../../services/auth";
import { showError } from "../../utils/error";

type AvatarSource = "url" | "qq" | "qqGroup";

const props = defineProps<{
  show: boolean;
  user: LightNovelUser | null;
}>();
const emit = defineEmits<{
  "update:show": [show: boolean];
  updated: [user: LightNovelUser];
}>();

const editingAvatar = ref(false);
const avatarSource = ref<AvatarSource>("url");
const avatarValue = ref("");
const saving = ref(false);
const avatarInput = ref<Input | null>(null);
const avatarError = ref("");
const dialogId = useId();
let opener: HTMLElement | null = null;
const avatarLabel = computed(() => avatarSource.value === "url" ? "图片地址" : avatarSource.value === "qq" ? "QQ 号" : "QQ 群号");
const rememberFocus = () => {
  opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
};
const focusDialog = () => {
  const dialog = document.getElementById(dialogId);
  // Varlet renders the dialog role on its popup wrapper, outside dialog attrs.
  dialog?.closest("[role='dialog']")?.setAttribute("aria-labelledby", `${dialogId}-title`);
  dialog?.querySelector<HTMLElement>("button")?.focus();
};
const restoreFocus = () => {
  if (opener?.isConnected) opener.focus({ preventScroll: true });
};
const trapFocus = (event: KeyboardEvent) => {
  if (event.key !== "Tab") return;
  const controls = Array.from(document.getElementById(dialogId)?.querySelectorAll<HTMLElement>(
    "button:not(:disabled), input:not(:disabled), [tabindex='0']",
  ) ?? []).filter((element) => element.getClientRects().length > 0);
  const first = controls[0];
  const last = controls[controls.length - 1];
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last?.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first?.focus();
  }
};
watch(editingAvatar, async (editing) => {
  avatarError.value = "";
  await nextTick();
  if (editing) avatarInput.value?.focus();
  else if (props.show) focusDialog();
});
watch(avatarValue, () => { avatarError.value = ""; });
watch([avatarError, editingAvatar], async () => {
  await nextTick();
  const input = document.getElementById(dialogId)?.querySelector("input");
  input?.setAttribute("aria-invalid", String(Boolean(avatarError.value)));
  const errorMessage = document.getElementById(dialogId)?.querySelector(".var-form-details__error-message");
  errorMessage?.setAttribute("id", `${dialogId}-error`);
  input?.setAttribute("aria-describedby", `${dialogId}-${avatarError.value ? "error" : "hint"}`);
});
const profileItems = computed(() => {
  const user = props.user;
  if (!user) return [];

  return [
    { label: "UID", value: String(user.Id) },
    { label: "昵称", value: user.UserName },
    { label: "邮箱", value: user.Email },
    { label: "邀请码", value: user.InviteCode },
    { label: "用户组", value: user.UserGroup },
    { label: "注册时间", value: formatDate(user.RegisterAt) },
    { label: "等级", value: user.Growth ? `Lv${user.Growth.Level}` : undefined },
    { label: "经验值", value: user.Growth?.Exp?.toLocaleString() },
    { label: "金币", value: user.Growth?.Coin?.toLocaleString() },
    { label: "连续签到", value: user.Growth ? `${user.Growth.SignStreak} 天` : undefined },
  ].filter((item): item is { label: string; value: string } => Boolean(item.value));
});

watch(
  () => props.show,
  async (show) => {
    if (!show) editingAvatar.value = false;
    else {
      await nextTick();
      focusDialog();
    }
  },
);

const close = () => {
  if (!saving.value) emit("update:show", false);
}

const beginAvatarEdit = () => {
  const avatar = props.user?.Avatar ?? "";
  const qqAvatarPrefix = "https://q.qlogo.cn/headimg_dl?spec=100&dst_uin=";
  const qqGroupMatch = avatar.match(/^https:\/\/p\.qlogo\.cn\/gh\/(\d+)\/\1\/100$/);

  if (avatar.startsWith(qqAvatarPrefix)) {
    avatarSource.value = "qq";
    avatarValue.value = avatar.slice(qqAvatarPrefix.length);
  } else if (qqGroupMatch) {
    avatarSource.value = "qqGroup";
    avatarValue.value = qqGroupMatch[1];
  } else {
    avatarSource.value = "url";
    avatarValue.value = avatar;
  }
  editingAvatar.value = true;
}

const changeAvatarSource = (source: AvatarSource) => {
  if (saving.value || avatarSource.value === source) return;
  avatarSource.value = source;
  avatarValue.value = "";
}

const avatarUrl = () => {
  const value = avatarValue.value.trim();
  if (avatarSource.value === "qq") {
    if (!/^[1-9]\d{4,}$/.test(value)) throw new Error("请输入正确的 QQ 号");
    return `https://q.qlogo.cn/headimg_dl?spec=100&dst_uin=${value}`;
  }
  if (avatarSource.value === "qqGroup") {
    if (!/^[1-9]\d{4,}$/.test(value)) throw new Error("请输入正确的 QQ 群号");
    return `https://p.qlogo.cn/gh/${value}/${value}/100`;
  }

  const url = new URL(value);
  if (url.protocol !== "https:") throw new Error("图片地址仅支持 HTTPS");
  return url.toString();
}

const saveAvatar = async () => {
  if (saving.value) return;
  let url: string;
  try {
    url = avatarUrl();
  } catch (error) {
    avatarError.value = error instanceof TypeError ? "请输入有效的 HTTPS 图片地址" : error instanceof Error ? error.message : "请检查头像信息";
    await nextTick();
    await avatarInput.value?.validate();
    avatarInput.value?.focus();
    return;
  }
  try {
    saving.value = true;
    const user = await setAvatar(url);
    emit("updated", user);
    editingAvatar.value = false;
    Snackbar.success("头像已更新");
  } catch (error) {
    showError(error, "更新头像失败");
  } finally {
    saving.value = false;
  }
}

const formatDate = (value?: string) => {
  if (!value) return undefined;
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : new Intl.DateTimeFormat("zh-CN", { dateStyle: "long" }).format(date);
}
</script>

<template>
  <var-dialog
    :id="dialogId"
    :show="show"
    :title="editingAvatar ? '修改头像' : '个人信息'"
    :aria-label="editingAvatar ? '修改头像' : '个人信息'"
    :close-on-click-overlay="!saving"
    :close-on-key-escape="!saving"
    :confirm-button="false"
    :cancel-button="false"
    dialog-class="profile-dialog"
    @update:show="emit('update:show', $event)"
    @open="rememberFocus"
    @opened="focusDialog"
    @closed="restoreFocus"
    @keydown="trapFocus"
  >
    <template #title><h2 :id="`${dialogId}-title`" class="profile-dialog__title">{{ editingAvatar ? "修改头像" : "个人信息" }}</h2></template>
    <section v-if="user && !editingAvatar" class="profile-dialog__content">
      <header class="profile-dialog__header">
        <var-avatar :size="64" :src="user.Avatar">
          <var-icon v-if="!user.Avatar" name="account-circle" />
        </var-avatar>
        <div class="profile-dialog__identity">
          <strong>{{ user.UserName }}</strong>
          <span>{{ user.UserGroup ?? "LightNovelShelf 用户" }}</span>
          <var-button text type="primary" :elevation="false" @click="beginAvatarEdit">更换头像</var-button>
        </div>
      </header>
      <dl class="profile-dialog__list">
        <div v-for="item in profileItems" :key="item.label" class="profile-dialog__item">
          <dt>{{ item.label }}</dt>
          <dd>{{ item.value }}</dd>
        </div>
      </dl>
    </section>

    <form v-else-if="user" :id="`${dialogId}-form`" class="profile-dialog__content" :aria-busy="saving" @submit.prevent="saveAvatar">
      <fieldset class="avatar-source-options">
        <legend>头像来源</legend>
        <var-radio-group :model-value="avatarSource" aria-label="头像来源" @update:model-value="changeAvatarSource">
          <var-radio checked-value="url" :disabled="saving">图片 URL</var-radio>
          <var-radio checked-value="qq" :disabled="saving">QQ 头像</var-radio>
          <var-radio checked-value="qqGroup" :disabled="saving">QQ 群头像</var-radio>
        </var-radio-group>
      </fieldset>
      <var-input
        ref="avatarInput"
        v-model="avatarValue"
        variant="outlined"
        :placeholder="avatarLabel"
        :aria-label="avatarLabel"
        :inputmode="avatarSource === 'url' ? 'url' : 'numeric'"
        :disabled="saving"
        :rules="[() => avatarError || true]"
      >
        <template #extra-message>
          <span v-if="!avatarError" :id="`${dialogId}-hint`">{{ avatarSource === "url" ? "请输入 HTTPS 图片地址。" : "使用此功能会公开对应的 QQ 号或 QQ 群号。" }}</span>
        </template>
      </var-input>
    </form>
    <template #actions>
      <div class="profile-dialog__actions">
        <template v-if="editingAvatar">
          <var-button text type="primary" :disabled="saving" :elevation="false" @click="editingAvatar = false">取消</var-button>
          <var-button text type="primary" native-type="submit" :form="`${dialogId}-form`" :elevation="false" :loading="saving" :disabled="saving">保存</var-button>
        </template>
        <var-button v-else text type="primary" :elevation="false" @click="close">关闭</var-button>
      </div>
    </template>
  </var-dialog>
</template>

<style scoped>
.profile-dialog__title {
  margin: 0;
  font: var(--type-headline-small);
}
.profile-dialog__content {
  display: grid;
  gap: var(--space-4);
}
.profile-dialog__header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}
.profile-dialog__identity {
  display: grid;
  min-width: 0;
  gap: var(--space-1);
  justify-items: start;
}
.profile-dialog__identity strong {
  color: var(--color-text);
  font: var(--type-title-large);
  overflow-wrap: anywhere;
}
.profile-dialog__identity > span,
.avatar-source-options legend {
  color: var(--color-text-muted);
  font: var(--type-body-medium);
}
.profile-dialog__identity .var-button {
  margin-left: calc(-1 * var(--space-3));
  min-height: 48px;
}
.profile-dialog__list {
  margin: 0;
}
.profile-dialog__item {
  display: grid;
  grid-template-columns: minmax(64px, 1fr) minmax(0, 2fr);
  gap: var(--space-3);
  padding-block: var(--space-2);
  font: var(--type-body-medium);
}
.profile-dialog__item + .profile-dialog__item {
  border-top: 1px solid var(--color-border);
}
.profile-dialog__list dt {
  color: var(--color-text-muted);
}
.profile-dialog__list dd {
  margin: 0;
  color: var(--color-text);
  overflow-wrap: anywhere;
  text-align: right;
}
.profile-dialog__actions {
  display: flex;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: var(--space-2);
  padding: var(--space-4) var(--space-4) var(--space-3);
}
.profile-dialog__actions .var-button {
  min-height: 48px;
}
.avatar-source-options {
  min-width: 0;
  margin: 0;
  padding: 0;
  border: 0;
}
.avatar-source-options legend {
  padding: 0;
  margin-bottom: var(--space-1);
}
.avatar-source-options .var-radio-group {
  display: flex;
  flex-wrap: wrap;
  gap: 0 var(--space-2);
}
:deep(.var-radio__wrap) {
  min-height: 48px;
}
:deep([role="radio"]:focus-visible) {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-control);
}
</style>
