import { createRouter, createWebHashHistory } from "vue-router";
import MainView from "../views/MainView.vue";
import MinimalTodoView from "../views/MinimalTodoView.vue";
import QuickCaptureView from "../views/QuickCaptureView.vue";
import TaskDetailWindowView from "../views/TaskDetailWindowView.vue";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", component: MainView },
    { path: "/minimal", component: MinimalTodoView },
    { path: "/task-detail/:id", component: TaskDetailWindowView },
    { path: "/quick-capture", component: QuickCaptureView },
  ],
});

export default router;
