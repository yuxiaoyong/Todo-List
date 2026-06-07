import { ElMessage } from "element-plus";
import { previewImages } from "hevue-img-preview/v3";
import { resolveAttachmentUrl } from "../api";

async function resolvePreviewUrl(src: string): Promise<string> {
  if (src.startsWith("local://attachment/")) {
    return resolveAttachmentUrl(src);
  }
  return src;
}

export async function showImagePreview(src: string) {
  try {
    previewImages(await resolvePreviewUrl(src));
  } catch (err) {
    console.error("Failed to preview image:", err);
    ElMessage.error("图片预览失败");
  }
}

export async function showImagePreviewList(urls: string[], index = 0) {
  if (!urls.length) return;
  try {
    const imgList = await Promise.all(urls.map((src) => resolvePreviewUrl(src)));
    previewImages({
      imgList,
      nowImgIndex: Math.max(0, Math.min(index, imgList.length - 1)),
    });
  } catch (err) {
    console.error("Failed to preview images:", err);
    ElMessage.error("图片预览失败");
  }
}
