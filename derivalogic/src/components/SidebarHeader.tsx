
import React from 'react';
import { SidebarHeader } from "@/components/ui/sidebar";

export function AppSidebarHeader() {
  return (
    <SidebarHeader className="bg-white border-b border-gray-200">
      <div className="flex items-center justify-center p-4">
        <h2 className="text-xl font-bold text-gray-900">DerivaLogic</h2>
      </div>
    </SidebarHeader>
  );
}
