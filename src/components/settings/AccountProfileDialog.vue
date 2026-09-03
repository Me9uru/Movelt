<script setup lang="ts">
import { Snackbar } from "@varlet/ui";
import { computed, ref, watch } from "vue";
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
const avatarPlaceholder = computed(() => {
  if (avatarSource.value === "qq") return "请输入 QQ 号";
  if (avatarSource.value === "qqGroup") return "请输入 QQ 群号";
  return "请输入 HTTPS 图片地址";
});

watch(
  () => props.show,
  (show) => {
    if (!show) editingAvatar.value = false;
  },
);

const close = () => {
  emit("update:show", false);
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
  if (avatarSource.value === source) return;
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
  try {
    saving.value = true;
    const user = await setAvatar(avatarUrl());
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
    :show="show"
    :title="editingAvatar ? '修改头像' : '个人信息'"
    :confirm-button="false"
    :cancel-button="false"
    dialog-class="profile-dialog"
    @update:show="emit('update:show', $event)"
  >
    <section v-if="user && !editingAvatar" class="profile-dialog__content">
      <header class="profile-dialog__header">
        <button class="profile-dialog__avatar" type="button" aria-label="修改头像" @click="beginAvatarEdit">
          <var-avatar :size="64" :src="user.Avatar" />
          <span>更换头像</span>
        </button>
        <div>
          <strong>{{ user.UserName }}</strong>
          <span>{{ user.UserGroup ?? "LightNovelShelf 用户" }}</span>
        </div>
      </header>
      <dl class="profile-dialog__list">
        <template v-for="item in profileItems" :key="item.label">
          <dt>{{ item.label }}</dt>
          <dd>{{ item.value }}</dd>
        </template>
      </dl>
      <div class="profile-dialog__actions">
        <var-button text type="primary" @click="close">关闭</var-button>
      </div>
    </section>

    <section v-else-if="user" class="profile-dialog__content">
      <div class="avatar-source-options" role="radiogroup" aria-label="头像来源">
        <var-button type="primary" :tonal="avatarSource === 'url'" :outline="avatarSource !== 'url'" :elevation="false" @click="changeAvatarSource('url')">图片 URL</var-button>
        <var-button type="primary" :tonal="avatarSource === 'qq'" :outline="avatarSource !== 'qq'" :elevation="false" @click="changeAvatarSource('qq')">QQ 头像</var-button>
        <var-button type="primary" :tonal="avatarSource === 'qqGroup'" :outline="avatarSource !== 'qqGroup'" :elevation="false" @click="changeAvatarSource('qqGroup')">QQ群头像</var-button>
      </div>
      <label class="avatar-input-label">
        {{ avatarSource === "url" ? "图片地址" : avatarSource === "qq" ? "QQ 号" : "QQ群号" }}
        <var-input v-model="avatarValue" variant="outlined" :placeholder="avatarPlaceholder" />
      </label>
      <p class="avatar-input-hint">
        {{ avatarSource === "url" ? "图片地址仅支持 HTTPS。" : "使用此功能会公开对应的 QQ 号或 QQ 群号。" }}
      </p>
      <div class="profile-dialog__actions">
        <var-button text :disabled="saving" @click="editingAvatar = false">取消</var-button>
        <var-button type="primary" :elevation="false" :loading="saving" @click="saveAvatar">保存</var-button>
      </div>
    </section>
  </var-dialog>
</template>

<style scoped>
.profile-dialog__content { display: grid; gap: var(--space-4); }
.profile-dialog__header { display: flex; align-items: center; gap: var(--space-3); }
.profile-dialog__header > div { display: grid; min-width: 0; gap: var(--space-1); }
.profile-dialog__header strong { overflow: hidden; color: var(--color-text-strong); font-size: 17px; text-overflow: ellipsis; white-space: nowrap; }
.profile-dialog__header span, .avatar-input-hint { color: var(--color-text-muted); font-size: 13px; }
.profile-dialog__avatar { display: grid; gap: var(--space-1); padding: 0; border: 0; color: var(--color-primary-deep); background: transparent; font: inherit; font-size: 12px; cursor: pointer; }
.profile-dialog__avatar:focus-visible { outline: 2px solid var(--color-primary); outline-offset: var(--space-1); }
.profile-dialog__list { display: grid; grid-template-columns: auto minmax(0, 1fr); gap: var(--space-2) var(--space-4); margin: 0; font-size: 14px; }
.profile-dialog__list dt { color: var(--color-text-muted); }
.profile-dialog__list dd { min-width: 0; margin: 0; overflow-wrap: anywhere; text-align: right; }
.profile-dialog__actions { display: flex; justify-content: flex-end; gap: var(--space-2); }
.avatar-source-options { display: flex; flex-wrap: wrap; gap: var(--space-2); }
.avatar-input-label { display: grid; gap: var(--space-1); color: var(--color-text-muted); font-size: 14px; }
.avatar-input-hint { margin: calc(var(--space-1) * -1) 0 0; line-height: 1.5; }
</style>
