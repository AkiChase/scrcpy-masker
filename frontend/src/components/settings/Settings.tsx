// TODO 设置可视化
// TODO 添加剪切板同步

import { useAppSelector } from "../../store/store";

// TODO 添加语言设置与后端同步
export default function Settings() {
  const localConfig = useAppSelector((state) => state.localConfig);

  return <div>Settings</div>;
}
