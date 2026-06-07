import Image, { type ImageOptions } from "@tiptap/extension-image";
import { VueNodeViewRenderer } from "@tiptap/vue-3";
import ResizableImageView from "./ResizableImageView.vue";

export interface ResizableImageOptions extends ImageOptions {
  onPreview?: (src: string) => void;
}

function parseWidth(element: HTMLElement): string | null {
  const styleWidth = element.style.width;
  if (styleWidth) return styleWidth;
  const attr = element.getAttribute("width");
  if (!attr) return null;
  return /^\d+$/.test(attr) ? `${attr}px` : attr;
}

export const ResizableImage = Image.extend<ResizableImageOptions>({
  addOptions() {
    return {
      ...this.parent?.(),
      onPreview: undefined,
    };
  },

  addAttributes() {
    return {
      ...this.parent?.(),
      width: {
        default: null,
        parseHTML: (element) => parseWidth(element),
        renderHTML: (attributes) => {
          if (!attributes.width) return {};
          return { style: `width: ${attributes.width}` };
        },
      },
    };
  },
  addNodeView() {
    return VueNodeViewRenderer(ResizableImageView);
  },
});
