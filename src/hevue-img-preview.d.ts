declare module "hevue-img-preview/v3" {
  import type { App, Plugin } from "vue";

  export interface HevueImgPreviewOptions {
    imgList?: string[];
    nowImgIndex?: number;
    thumbnail?: boolean;
    controlBar?: string[];
    closeBtn?: boolean;
    arrowBtn?: boolean;
    clickMaskCLose?: boolean;
    disabledImgRightClick?: boolean;
    disableTransition?: boolean;
    customStyle?: Record<string, unknown>;
    themeName?: string;
    zIndex?: number;
    closeFn?: () => void;
  }

  export function previewImages(
    options: string | string[] | HevueImgPreviewOptions,
  ): unknown;

  const plugin: Plugin<[HevueImgPreviewOptions?]>;
  export default plugin;
}
