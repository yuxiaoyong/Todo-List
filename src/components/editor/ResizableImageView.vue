<script setup lang="ts">
import { computed, ref } from "vue";
import { NodeViewWrapper } from "@tiptap/vue-3";
import type { NodeViewProps } from "@tiptap/core";
import type { ResizableImageOptions } from "./resizableImage";

const props = defineProps<NodeViewProps>();

const imgRef = ref<HTMLImageElement | null>(null);
const wrapperRef = ref<HTMLElement | null>(null);
const resizing = ref(false);

const width = computed(() => props.node.attrs.width as string | null | undefined);

const imgStyle = computed(() => {
  const style: Record<string, string> = {
    maxWidth: "100%",
    height: "auto",
    display: "block",
  };
  if (width.value) {
    style.width = width.value;
  }
  return style;
});

function selectImage() {
  const pos = props.getPos();
  if (typeof pos !== "number") return;
  props.editor.chain().focus().setNodeSelection(pos).run();
}

function onDoubleClick(event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  const src = props.node.attrs.src as string | null;
  if (!src) return;
  const options = props.extension.options as ResizableImageOptions;
  options.onPreview?.(src);
}

function onResizeStart(event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  const img = imgRef.value;
  const wrapper = wrapperRef.value;
  if (!img || !wrapper) return;

  const startX = event.clientX;
  const startWidth = img.getBoundingClientRect().width;
  const maxWidth = wrapper.parentElement?.clientWidth ?? startWidth;

  resizing.value = true;

  const onMove = (moveEvent: MouseEvent) => {
    const nextWidth = Math.round(
      Math.max(80, Math.min(startWidth + (moveEvent.clientX - startX), maxWidth)),
    );
    props.updateAttributes({ width: `${nextWidth}px` });
  };

  const onUp = () => {
    resizing.value = false;
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  };

  document.body.style.cursor = "nwse-resize";
  document.body.style.userSelect = "none";
  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
}
</script>

<template>
  <NodeViewWrapper
    as="div"
    class="resizable-image"
    :class="{ 'is-selected': selected, 'is-resizing': resizing }"
    contenteditable="false"
  >
    <div
      ref="wrapperRef"
      class="resizable-image__wrapper"
      @click="selectImage"
      @dblclick="onDoubleClick"
    >
      <img
        ref="imgRef"
        :src="node.attrs.src"
        :alt="node.attrs.alt ?? undefined"
        title="双击预览"
        :style="imgStyle"
        draggable="false"
        @dblclick="onDoubleClick"
      />
      <span
        v-if="selected && editor.isEditable"
        class="resizable-image__handle"
        title="拖动调整大小"
        @mousedown="onResizeStart"
      />
    </div>
  </NodeViewWrapper>
</template>

<style scoped>
.resizable-image {
  display: block;
  margin: 0.5em 0;
}

.resizable-image__wrapper {
  position: relative;
  display: inline-block;
  max-width: 100%;
  line-height: 0;
}

.resizable-image__wrapper img {
  border-radius: 4px;
  cursor: zoom-in;
}

.resizable-image.is-selected .resizable-image__wrapper {
  outline: 2px solid var(--el-color-primary, #1677ff);
  outline-offset: 2px;
  border-radius: 4px;
}

.resizable-image__handle {
  position: absolute;
  right: -4px;
  bottom: -4px;
  width: 12px;
  height: 12px;
  background: var(--el-color-primary, #1677ff);
  border: 2px solid #fff;
  border-radius: 2px;
  cursor: nwse-resize;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.resizable-image.is-resizing .resizable-image__handle {
  background: var(--el-color-primary-dark-2, #0958d9);
}
</style>
