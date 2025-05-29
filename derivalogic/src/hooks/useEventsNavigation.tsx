
import { useCallback } from 'react';
import { useNavigate } from 'react-router-dom';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

interface EventsNavigationProps {
  hasUnsavedChanges: boolean;
  isDraftWithChanges: () => boolean;
  setPendingNavigation: (nav: (() => void) | null) => void;
  setShowUnsavedChangesModal: (show: boolean) => void;
  createScriptMutation: any;
  script?: Script;
}

export function useEventsNavigation({
  hasUnsavedChanges,
  isDraftWithChanges,
  setPendingNavigation,
  setShowUnsavedChangesModal,
  createScriptMutation,
  script
}: EventsNavigationProps) {
  const navigate = useNavigate();

  const checkUnsavedChanges = useCallback((callback: () => void) => {
    // Only show modal for saved scripts with unsaved changes, not drafts
    if (hasUnsavedChanges && script?.status === 'SAVED') {
      setPendingNavigation(() => callback);
      setShowUnsavedChangesModal(true);
    } else {
      callback();
    }
  }, [hasUnsavedChanges, script?.status, setPendingNavigation, setShowUnsavedChangesModal]);

  const handleScriptSelect = useCallback((selectedScript: Script) => {
    checkUnsavedChanges(() => {
      navigate('/events', { state: { script: selectedScript } });
    });
  }, [checkUnsavedChanges, navigate]);

  const handleNewScript = useCallback(() => {
    checkUnsavedChanges(() => {
      createScriptMutation.mutate();
    });
  }, [checkUnsavedChanges, createScriptMutation]);

  return {
    checkUnsavedChanges,
    handleScriptSelect,
    handleNewScript
  };
}
