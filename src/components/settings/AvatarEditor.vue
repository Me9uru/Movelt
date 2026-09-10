<script setup lang="ts">
import { Input, Snackbar } from "@varlet/ui";
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { useAuthStore } from "../../stores/auth";
import { showError } from "../../utils/error";
type AvatarSource = "url" | "qq" | "qqGroup";
const props = defineProps<{ avatar: string; formId: string; }>();
const emit = defineEmits<{ saved: []; }>();
const saving = defineModel<boolean>("saving", { required: true });
const auth = useAuthStore();
const avatarSource = ref<AvatarSource>("url");
const avatarValue = ref("");
const avatarError = ref("");
const avatarInput = ref<Input | null>(null);
const form = ref<HTMLFormElement | null>(null);
const avatarLabel = computed(() => avatarSource.value === "url" ? "图片地址" : avatarSource.value === "qq" ? "QQ 号" : "QQ 群号");
const avatar = props.avatar;
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

onMounted(() => avatarInput.value?.focus());
watch(avatarValue, () => { avatarError.value = ""; });
watch(avatarError, async () => {
  await nextTick();
  const input = form.value?.querySelector("input");
  input?.setAttribute("aria-invalid", String(Boolean(avatarError.value)));
  const errorMessage = form.value?.querySelector(".var-form-details__error-message");
  errorMessage?.setAttribute("id", `${props.formId}-error`);
  input?.setAttribute("aria-describedby", `${props.formId}-${avatarError.value ? "error" : "hint"}`);
}, { immediate: true });

const changeAvatarSource = (source: AvatarSource) => {
  if (saving.value || avatarSource.value === source) return;
  avatarSource.value = source;
  avatarValue.value = "";
};

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
};

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
    await auth.updateAvatar(url);
    emit("saved");
    Snackbar.success("头像已更新");
  } catch (error) {
    showError(error, "更新头像失败");
  } finally {
    saving.value = false;
  }
}


</script>

<template>
    <form ref="form" :id="formId" class="profile-dialog__content" :aria-busy="saving" @submit.prevent="saveAvatar">
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
          <span v-if="!avatarError" :id="`${formId}-hint`">{{ avatarSource === "url" ? "请输入 HTTPS 图片地址。" : "使用此功能会公开对应的 QQ 号或 QQ 群号。" }}</span>
        </template>
      </var-input>
    </form>
</template>

<style scoped>
.avatar-source-options {
  min-width: 0;
  margin: 0;
  padding: 0;
  border: 0;
}
.avatar-source-options legend {
  color: var(--color-text-muted);
  font: var(--type-body-medium);
  padding: 0;
  margin-bottom: var(--space-1);
}
.avatar-source-options .var-radio-group {
  display: flex;
  flex-wrap: wrap;
  gap: 0 var(--space-2);
}
:deep(.var-radio__wrap) { min-height: 48px; }
:deep([role="radio"]:focus-visible) {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-control);
}
</style>
