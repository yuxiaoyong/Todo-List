<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { TodoSummary } from "../../types";

defineProps<{ todo: TodoSummary }>();
const emit = defineEmits<{ toggle: []; click: [] }>();
const { t } = useI18n();
</script>

<template>
  <div class="todo-item" :class="{ done: todo.completed }" @click="emit('click')">
    <el-checkbox :model-value="todo.completed" @click.stop @change="emit('toggle')" />
    <div class="body">
      <div class="title-row">
        <span class="title">{{ todo.title }}</span>
        <el-tag v-if="todo.priority === 'high'" size="small" type="danger">{{
          t("priority.high")
        }}</el-tag>
        <el-tag v-else-if="todo.priority === 'low'" size="small" type="info">{{
          t("priority.low")
        }}</el-tag>
      </div>
      <div v-if="todo.dueDate" class="meta">
        {{ t("task.duePrefix") }} {{ todo.dueDate }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.todo-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-color);
  cursor: pointer;
}
.todo-item.done .title {
  text-decoration: line-through;
  color: var(--text-secondary);
}
.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.meta {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}
</style>
