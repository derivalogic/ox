
import { useState, useEffect, useRef } from 'react';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

export function useEventsState(script: Script | null) {
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const [isSaveModalOpen, setIsSaveModalOpen] = useState(false);
  const [showUnsavedChangesModal, setShowUnsavedChangesModal] = useState(false);
  const [pendingNavigation, setPendingNavigation] = useState<(() => void) | null>(null);
  const [isModifyMode, setIsModifyMode] = useState(false);
  const [showSavedMessage, setShowSavedMessage] = useState(false);
  const [currentScriptName, setCurrentScriptName] = useState(script?.name || 'Untitled');
  const lastScriptIdRef = useRef<string | null>(null);

  // Reset state when script changes
  useEffect(() => {
    if (script && script.id !== lastScriptIdRef.current) {
      lastScriptIdRef.current = script.id;
      
      // Reset unsaved changes when switching to a different script
      setHasUnsavedChanges(false);
      setShowSavedMessage(false);
      
      // Set modify mode based on script status
      if (script.status === 'DRAFT') {
        setIsModifyMode(true);
        // Mark drafts as having unsaved changes
        setHasUnsavedChanges(true);
      } else {
        // For saved scripts, start in view mode
        setIsModifyMode(false);
      }
      
      // Update script name
      console.log('Script name updated from script prop:', script.name);
      setCurrentScriptName(script.name);
    }
  }, [script?.id, script?.name, script?.status]);

  // Check for unsaved changes before page unload
  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (hasUnsavedChanges || (script?.status === 'DRAFT' && isModifyMode)) {
        e.preventDefault();
        e.returnValue = 'You have unsaved changes. Are you sure you want to leave?';
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => window.removeEventListener('beforeunload', handleBeforeUnload);
  }, [hasUnsavedChanges, script?.status, isModifyMode]);

  const markUnsavedChanges = () => setHasUnsavedChanges(true);
  
  const markSaved = (scriptName: string) => {
    console.log('markSaved called with scriptName:', scriptName);
    setHasUnsavedChanges(false);
    setIsModifyMode(false);
    setShowSavedMessage(true);
    setCurrentScriptName(scriptName);
    // Hide the saved message after 3 seconds
    setTimeout(() => setShowSavedMessage(false), 3000);
  };

  const enterModifyMode = () => {
    setIsModifyMode(true);
    setShowSavedMessage(false);
  };

  const isDraftWithChanges = () => {
    return script?.status === 'DRAFT' && (hasUnsavedChanges || isModifyMode);
  };

  // Function to update current script name
  const updateCurrentScriptName = (name: string) => {
    console.log('updateCurrentScriptName called with:', name);
    setCurrentScriptName(name);
    // Mark as having unsaved changes when name is modified
    if (script?.status !== 'DRAFT') {
      setHasUnsavedChanges(true);
    }
  };

  return {
    hasUnsavedChanges,
    isSaveModalOpen,
    setIsSaveModalOpen,
    showUnsavedChangesModal,
    setShowUnsavedChangesModal,
    pendingNavigation,
    setPendingNavigation,
    isModifyMode,
    setIsModifyMode: enterModifyMode,
    showSavedMessage,
    setShowSavedMessage,
    currentScriptName,
    setCurrentScriptName: updateCurrentScriptName,
    markUnsavedChanges,
    markSaved,
    isDraftWithChanges
  };
}
