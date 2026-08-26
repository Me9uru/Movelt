<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";

const props = withDefaults(
  defineProps<{
    hasPrevious?: boolean;
    hasNext?: boolean;
    disabled?: boolean;
    buffer?: number;
  }>(),
  {
    hasPrevious: false,
    hasNext: false,
    disabled: false,
    buffer: 96,
  },
);

const emit = defineEmits<{
  previous: [];
  next: [];
}>();

const boundaryRoot = ref<HTMLElement | null>(null);
const pendingDirection = ref<"previous" | "next" | null>(null);
const pendingTrigger = ref<"touch" | "wheel" | null>(null);
let pointer: { id: number; startY: number } | null = null;
let touch: { id: number; startY: number } | null = null;
let lastScrollY = 0;
let wheelDirection = 0;
let wheelTimer: number | null = null;

function readerBounds(): { top: number; bottom: number } | null {
  const reader = boundaryRoot.value?.closest<HTMLElement>(".book-reader");
  if (!reader) return null;
  const top = reader.getBoundingClientRect().top + window.scrollY;
  return {
    top,
    bottom: top + reader.scrollHeight - window.innerHeight,
  };
}

function directionAtBoundary(
  scrollDirection: number,
): "previous" | "next" | null {
  const bounds = readerBounds();
  if (!bounds) return null;
  const topDistance = window.scrollY - bounds.top;
  const bottomDistance = bounds.bottom - window.scrollY;
  if (scrollDirection < 0 && topDistance <= props.buffer && props.hasPrevious)
    return "previous";
  if (scrollDirection > 0 && bottomDistance <= props.buffer && props.hasNext)
    return "next";
  return null;
}

function clearPending(): void {
  pendingDirection.value = null;
  pendingTrigger.value = null;
}

function setPending(
  direction: "previous" | "next" | null,
  trigger: "touch" | "wheel",
): void {
  pendingDirection.value = direction;
  pendingTrigger.value = direction ? trigger : null;
}

function handleScroll(): void {
  const direction = window.scrollY - lastScrollY;
  lastScrollY = window.scrollY;
  if (props.disabled || direction === 0) return;
  if (pointer || touch) {
    setPending(directionAtBoundary(direction), "touch");
  } else if (wheelTimer !== null) {
    setPending(directionAtBoundary(direction || wheelDirection), "wheel");
  }
}

function handlePointerDown(event: PointerEvent): void {
  // Touch is handled through the native TouchEvent path below. In particular,
  // some Android WebViews deliver incomplete PointerEvent sequences while the
  // page is scrolling.
  if (event.pointerType !== "pen" || props.disabled) return;
  pointer = { id: event.pointerId, startY: event.clientY };
  if (wheelTimer !== null) window.clearTimeout(wheelTimer);
  wheelTimer = null;
  if (event.currentTarget instanceof HTMLElement)
    event.currentTarget.setPointerCapture(event.pointerId);
  lastScrollY = window.scrollY;
  clearPending();
}

function changedTouch(event: TouchEvent, identifier: number): Touch | null {
  return (
    [...event.changedTouches].find((item) => item.identifier === identifier) ??
    null
  );
}

function handleTouchStart(event: TouchEvent): void {
  if (props.disabled || event.changedTouches.length !== 1) return;
  const start = event.changedTouches[0];
  if (!start) return;
  touch = { id: start.identifier, startY: start.clientY };
  if (wheelTimer !== null) window.clearTimeout(wheelTimer);
  wheelTimer = null;
  lastScrollY = window.scrollY;
  clearPending();
}

function handleTouchMove(event: TouchEvent): void {
  if (!touch || props.disabled) return;
  const current = changedTouch(event, touch.id);
  if (!current) return;
  const fingerDistance = current.clientY - touch.startY;
  if (Math.abs(fingerDistance) < 24) return;
  setPending(directionAtBoundary(-fingerDistance), "touch");
}

function completeTouch(event: TouchEvent, cancelled = false): void {
  if (!touch || !changedTouch(event, touch.id)) return;
  touch = null;
  const direction = pendingDirection.value;
  const trigger = pendingTrigger.value;
  clearPending();
  if (cancelled || props.disabled || trigger !== "touch") return;
  if (direction === "previous") emit("previous");
  if (direction === "next") emit("next");
}

function handlePointerMove(event: PointerEvent): void {
  if (!pointer || event.pointerId !== pointer.id || props.disabled) return;
  const fingerDistance = event.clientY - pointer.startY;
  if (Math.abs(fingerDistance) < 24) return;
  // At an exact edge, native overscroll may not produce another scroll event.
  // Use the drag direction so the confirmation affordance remains available.
  setPending(directionAtBoundary(-fingerDistance), "touch");
}

function handlePointerEnd(event: PointerEvent): void {
  if (!pointer || event.pointerId !== pointer.id) return;
  pointer = null;
  const direction = pendingDirection.value;
  clearPending();
  if (props.disabled) return;
  if (direction === "previous") emit("previous");
  if (direction === "next") emit("next");
}

function handlePointerCancel(event: PointerEvent): void {
  if (!pointer || event.pointerId !== pointer.id) return;
  pointer = null;
  clearPending();
}

function handleWheel(event: WheelEvent): void {
  if (props.disabled || event.deltaY === 0) return;
  wheelDirection = event.deltaY;
  if (wheelTimer !== null) window.clearTimeout(wheelTimer);
  setPending(directionAtBoundary(wheelDirection), "wheel");
  wheelTimer = window.setTimeout(() => {
    wheelTimer = null;
    const direction = pendingDirection.value;
    const trigger = pendingTrigger.value;
    clearPending();
    if (props.disabled || trigger !== "wheel") return;
    if (direction === "previous") emit("previous");
    if (direction === "next") emit("next");
  }, 200);
}

watch(() => [props.disabled, props.hasPrevious, props.hasNext], clearPending);

onMounted(() => {
  lastScrollY = window.scrollY;
  window.addEventListener("scroll", handleScroll, { passive: true });
});

onBeforeUnmount(() => {
  window.removeEventListener("scroll", handleScroll);
  if (wheelTimer !== null) window.clearTimeout(wheelTimer);
});
</script>

<template>
  <div
    ref="boundaryRoot"
    class="reader-boundary-switch"
    @pointerdown.passive="handlePointerDown"
    @pointermove.passive="handlePointerMove"
    @pointerup="handlePointerEnd"
    @pointercancel="handlePointerCancel"
    @touchstart.capture.passive="handleTouchStart"
    @touchmove.capture.passive="handleTouchMove"
    @touchend.capture="completeTouch($event)"
    @touchcancel.capture="completeTouch($event, true)"
    @wheel.passive="handleWheel"
  >
    <slot />
    <Transition name="reader-boundary-hint">
      <p
        v-if="pendingDirection"
        class="reader-boundary-hint"
        :class="`reader-boundary-hint--${pendingDirection}`"
        role="status"
      >
        {{ pendingTrigger === "wheel" ? "停止滚动时" : "松手时" }}加载{{
          pendingDirection === "previous" ? "上一话" : "下一话"
        }}
      </p>
    </Transition>
  </div>
</template>
