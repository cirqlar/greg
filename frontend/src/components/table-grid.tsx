import { ReactNode } from "react";

import style from "./table-grid.module.css";

function TableGrid({
  children,
  className = style.template_4,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div className=" overflow-y-auto">
      <div
        className={`max-w-full min-w-[500px] grid gap-x-4 gap-y-4 ${className}`}
      >
        {children}
      </div>
    </div>
  );
}

export default TableGrid;
