
import React, { useState, useEffect } from 'react';
import { 
  Sidebar, 
  SidebarContent, 
  SidebarGroup,
  SidebarGroupLabel,
  SidebarGroupContent,
  SidebarMenu, 
  SidebarMenuButton, 
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { 
  Plus, 
  SettingsIcon,
  Home,
  MessageSquare,
} from "lucide-react";
import { useNavigate } from 'react-router-dom';
import { AppSidebarHeader } from "@/components/SidebarHeader";
import { AppSidebarFooter } from "@/components/SidebarFooter";
import { SidebarScriptsList } from "@/components/SidebarScriptsList";

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

interface AppSidebarProps {
  scripts?: Script[];
  currentScript?: Script;
  onScriptSelect?: (script: Script) => void;
  onNewScript?: () => void;
  onDeleteDraft?: (script: Script) => void;
  onSaveScript?: (script: Script) => void;
}

export function AppSidebar({ 
  scripts = [], 
  currentScript, 
  onScriptSelect, 
  onNewScript,
  onDeleteDraft,
  onSaveScript
}: AppSidebarProps) {
  const [openScripts, setOpenScripts] = useState<string[]>([]);
  const navigate = useNavigate();

  // Keep current script expanded when it changes
  useEffect(() => {
    if (currentScript?.id && !openScripts.includes(currentScript.id)) {
      setOpenScripts(prev => [...prev, currentScript.id]);
    }
  }, [currentScript?.id]);

  const toggleScript = (scriptId: string) => {
    setOpenScripts(prev => 
      prev.includes(scriptId) 
        ? prev.filter(id => id !== scriptId)
        : [...prev, scriptId]
    );
  };

  const handleHomeClick = () => {
    navigate('/');
  };

  const handleFeedbackClick = () => {
    navigate('/feedback');
  };

  return (
    <Sidebar className="bg-white border-r border-gray-200">
      <AppSidebarHeader />
      <SidebarContent className="bg-white">
        <SidebarGroup>
          <SidebarGroupLabel className="text-gray-700 font-medium">General</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton onClick={handleHomeClick} className="text-gray-700 hover:bg-gray-100">
                  <Home className="h-4 w-4" />
                  <span>Home</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton onClick={onNewScript} className="text-gray-700 hover:bg-gray-100">
                  <Plus className="h-4 w-4" />
                  <span>New Script</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton onClick={handleFeedbackClick} className="text-gray-700 hover:bg-gray-100">
                  <MessageSquare className="h-4 w-4" />
                  <span>Feedback</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton className="text-gray-700 hover:bg-gray-100">
                  <SettingsIcon className="h-4 w-4" />
                  <span>Global Settings</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarScriptsList
          scripts={scripts}
          currentScript={currentScript}
          openScripts={openScripts}
          onToggleScript={toggleScript}
          onScriptSelect={onScriptSelect}
          onDeleteDraft={onDeleteDraft}
          onSaveScript={onSaveScript}
        />
      </SidebarContent>
      <AppSidebarFooter />
    </Sidebar>
  );
}
