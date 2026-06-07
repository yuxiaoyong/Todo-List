<script setup lang="ts">
import { computed, onUnmounted, ref } from "vue";
import { useDraggable } from "vue-draggable-plus";
import type { SortableEvent } from "sortablejs";
import type { TodoSummary } from "../../types";

const props = defineProps<{
  bucketKey: string;
  modelValue: TodoSummary[];
  disabled?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [TodoSummary[]];
  sortStart: [bucketKey: string];
  sortEnd: [evt: SortableEvent, bucketKey: string];
}>();

const containerEl = ref<HTMLElement | null>(null);

const list = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const dragOptions = computed(() => ({
  group: { name: "kanban-todos", pull: true, put: true },
  animation: 220,
  easing: "cubic-bezier(0.2, 0, 0, 1)",
  ghostClass: "drag-ghost",
  chosenClass: "drag-chosen",
  dragClass: "dragging-card",
  draggable: ".task-card",
  filter: ".no-drag",
  preventOnFilter: false,
  forceFallback: true,
  fallbackOnBody: true,
  swapThreshold: 0.65,
  emptyInsertThreshold: 16,
  disabled: props.disabled ?? false,
  onStart() {
    emit("sortStart", props.bucketKey);
  },
  onEnd(evt: SortableEvent) {
    document
      .querySelectorAll("body > .sortable-fallback, body > .dragging-card")
      .forEach((node) => node.remove());
    emit("sortEnd", evt, props.bucketKey);
  },
}));

const draggable = useDraggable(containerEl, list, dragOptions);

onUnmounted(() => {
  draggable.destroy?.();
});
</script>

<template>
  <div
    ref="containerEl"
    class="column-body"
    :class="{ 'is-empty': !modelValue.length }"
    :data-bucket="bucketKey"
  >
    <template v-for="todo in modelValue" :key="todo.id">
      <slot :todo="todo" />
    </template>
  </div>
</template>
