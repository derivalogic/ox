
import React from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import {
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarMenuSub,
  SidebarMenuSubItem,
  SidebarMenuSubButton,
} from "@/components/ui/sidebar";
import { 
  FolderIcon, 
  Trash2,
  Save
} from "lucide-react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { ChevronDown, ChevronRight } from "lucide-react";
import { useToast } from '@/hooks/use-toast';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

interface SidebarScriptItemProps {
  script: Script;
  isDraft?: boolean;
  isOpen: boolean;
  currentScript?: Script;
  onToggle: () => void;
  onScriptSelect?: (script: Script) => void;
  onDeleteDraft?: (script: Script) => void;
  onSaveScript?: (script: Script) => void;
}

export function SidebarScriptItem({
  script,
  isDraft = false,
  isOpen,
  currentScript,
  onToggle,
  onScriptSelect,
  onDeleteDraft,
  onSaveScript
}: SidebarScriptItemProps) {
  const navigate = useNavigate();
  const location = useLocation();
  const { toast } = useToast();

  const handleScriptAction = (script: Script, action: string) => {
    if (onScriptSelect) {
      onScriptSelect(script);
    }
    
    if (action === 'Settings') {
      if (script.status === 'DRAFT') {
        toast({ 
          title: "Cannot access settings", 
          description: "Please save the script first before accessing settings.",
          variant: "destructive"
        });
        return;
      }
      navigate('/script-settings', { state: { script } });
    } else if (action === 'Events') {
      navigate('/events', { state: { script } });
    } else if (action === 'Transaction Details') {
      if (script.status === 'DRAFT') {
        toast({ 
          title: "Cannot access transaction details", 
          description: "Please save the script first before accessing transaction details.",
          variant: "destructive"
        });
        return;
      }
      navigate('/transaction-details', { state: { script } });
    }
  };

  const handleDeleteDraft = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onDeleteDraft) {
      onDeleteDraft(script);
    }
  };

  const handleSaveScript = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onSaveScript) {
      onSaveScript(script);
    }
  };

  return (
    <SidebarMenuItem>
      <Collapsible open={isOpen} onOpenChange={onToggle}>
        <CollapsibleTrigger asChild>
          <SidebarMenuButton
            className={`text-gray-700 hover:bg-gray-100 ${currentScript?.id === script.id ? "bg-gray-100" : ""}`}
          >
            <FolderIcon className="h-4 w-4" />
            <span className="truncate flex items-center gap-2 flex-1">
              {script.name || 'Untitled'}
              {isDraft && (
                <span className="bg-yellow-500/20 text-yellow-600 px-1.5 py-0.5 rounded text-xs font-medium">
                  DRAFT
                </span>
              )}
            </span>
            <div className="flex items-center gap-1">
              {isDraft && (
                <>
                  <div
                    onClick={handleSaveScript}
                    className="p-1 hover:bg-green-100 rounded text-green-600 hover:text-green-700 cursor-pointer"
                    title="Save script"
                  >
                    <Save className="h-3 w-3" />
                  </div>
                  <div
                    onClick={handleDeleteDraft}
                    className="p-1 hover:bg-red-100 rounded text-red-600 hover:text-red-700 cursor-pointer"
                    title="Delete draft"
                  >
                    <Trash2 className="h-3 w-3" />
                  </div>
                </>
              )}
              {isOpen ? 
                <ChevronDown className="h-4 w-4" /> : 
                <ChevronRight className="h-4 w-4" />
              }
            </div>
          </SidebarMenuButton>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <SidebarMenuSub>
            <SidebarMenuSubItem>
              <SidebarMenuSubButton 
                onClick={() => handleScriptAction(script, 'Events')}
                className={`text-gray-600 hover:bg-gray-50 ${location.pathname === '/events' && currentScript?.id === script.id ? "bg-gray-100" : ""}`}
              >
                Events
              </SidebarMenuSubButton>
            </SidebarMenuSubItem>
            <SidebarMenuSubItem>
              <SidebarMenuSubButton 
                onClick={() => handleScriptAction(script, 'Transaction Details')}
                className={`text-gray-600 hover:bg-gray-50 ${script.status === 'DRAFT' ? 'opacity-50 cursor-not-allowed' : ''} ${location.pathname === '/transaction-details' && currentScript?.id === script.id ? "bg-gray-100" : ""}`}
              >
                Transaction Details
              </SidebarMenuSubButton>
            </SidebarMenuSubItem>
            <SidebarMenuSubItem>
              <SidebarMenuSubButton 
                onClick={() => handleScriptAction(script, 'Settings')}
                className={`text-gray-600 hover:bg-gray-50 ${script.status === 'DRAFT' ? 'opacity-50 cursor-not-allowed' : ''} ${location.pathname === '/script-settings' && currentScript?.id === script.id ? "bg-gray-100" : ""}`}
              >
                Settings
              </SidebarMenuSubButton>
            </SidebarMenuSubItem>
          </SidebarMenuSub>
        </CollapsibleContent>
      </Collapsible>
    </SidebarMenuItem>
  );
}
