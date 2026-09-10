<script setup lang="ts">
import { computed, nextTick, ref, useId, watch } from "vue";
import type { LightNovelUser } from "../../domain/auth";
import AvatarEditor from "./AvatarEditor.vue";
import { useDialogFocus } from "../../composables/useDialogFocus";


const props = defineProps<{
  show: boolean;
  user: LightNovelUser | null;
}>();
const emit = defineEmits<{
  "update:show": [show: boolean];
}>();

const editingAvatar = ref(false);
const saving = ref(false);
const dialogId = useId();
const { rememberFocus, restoreFocus, trapFocus } = useDialogFocus(() => document.getElementById(dialogId));
const focusDialog = () => {
  const dialog = document.getElementById(dialogId);
  // Varlet renders the dialog role on its popup wrapper, outside dialog attrs.
  dialog?.closest("[role='dialog']")?.setAttribute("aria-labelledby", `${dialogId}-title`);
  dialog?.querySelector<HTMLElement>(editingAvatar.value ? "input" : "button")?.focus();
};
watch(editingAvatar, async (editing) => {
  if (!editing && props.show) { await nextTick(); focusDialog(); }
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
  ].filter((item): item is { label: string; value: string; } => Boolean(item.value));
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
};

const formatDate = (value?: string) => {
  if (!value) return undefined;
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : new Intl.DateTimeFormat("zh-CN", { dateStyle: "long" }).format(date);
};
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
          <var-button text type="primary" :elevation="false" @click="editingAvatar = true">更换头像</var-button>
        </div>
      </header>
      <dl class="profile-dialog__list">
        <div v-for="item in profileItems" :key="item.label" class="profile-dialog__item">
          <dt>{{ item.label }}</dt>
          <dd>{{ item.value }}</dd>
        </div>
      </dl>
    </section>

    <AvatarEditor
      v-else-if="user"
      v-model:saving="saving"
      :avatar="user.Avatar ?? ''"
      :form-id="`${dialogId}-form`"
      class="profile-dialog__content"
      @saved="editingAvatar = false"
    />
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
.profile-dialog__identity > span {
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
</style>
