
import React from 'react';
import { SidebarFooter } from "@/components/ui/sidebar";
import { UserProfile } from "@/components/UserProfile";

export function AppSidebarFooter() {
  return (
    <SidebarFooter className="bg-white border-t border-gray-200">
      <div className="flex items-center justify-center p-2">
        <UserProfile />
      </div>
    </SidebarFooter>
  );
}
