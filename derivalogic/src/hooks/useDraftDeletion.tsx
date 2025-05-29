
import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { supabase } from '@/integrations/supabase/client';
import { useToast } from '@/hooks/use-toast';
import { useAuth } from '@/hooks/useAuth';
import { useNavigate } from 'react-router-dom';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

export function useDraftDeletion(currentScript?: Script) {
  const [showUnsavedChangesModal, setShowUnsavedChangesModal] = useState(false);
  const [scriptToDelete, setScriptToDelete] = useState<Script | null>(null);
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const { user } = useAuth();
  const navigate = useNavigate();

  // Delete draft script mutation
  const deleteDraftMutation = useMutation({
    mutationFn: async (scriptId: string) => {
      if (!user?.id) throw new Error('User not authenticated');
      
      // First delete all events associated with the script
      const { error: eventsError } = await supabase
        .from('events')
        .delete()
        .eq('script_id', scriptId)
        .eq('user_id', user.id);

      if (eventsError) throw eventsError;

      // Then delete the script
      const { error: scriptError } = await supabase
        .from('scripts')
        .delete()
        .eq('id', scriptId)
        .eq('user_id', user.id)
        .eq('status', 'DRAFT'); // Only delete if it's still a draft

      if (scriptError) throw scriptError;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scripts'] });
      toast({ 
        title: "Draft deleted successfully",
        description: "The draft script has been removed."
      });
      
      // If we deleted the current script, navigate to home
      if (scriptToDelete?.id === currentScript?.id) {
        navigate('/');
      }
      
      setScriptToDelete(null);
    },
    onError: (error) => {
      console.error('Error deleting draft script:', error);
      toast({ 
        title: "Error deleting draft", 
        description: "Failed to delete the draft script.",
        variant: "destructive" 
      });
      setScriptToDelete(null);
    }
  });

  const handleDeleteDraft = (script: Script, hasUnsavedChanges: boolean = false) => {
    setScriptToDelete(script);
    
    if (hasUnsavedChanges && script.id === currentScript?.id) {
      setShowUnsavedChangesModal(true);
    } else {
      deleteDraftMutation.mutate(script.id);
    }
  };

  const confirmDeleteWithUnsavedChanges = () => {
    setShowUnsavedChangesModal(false);
    if (scriptToDelete) {
      deleteDraftMutation.mutate(scriptToDelete.id);
    }
  };

  const cancelDeleteWithUnsavedChanges = () => {
    setShowUnsavedChangesModal(false);
    setScriptToDelete(null);
  };

  return {
    showUnsavedChangesModal,
    handleDeleteDraft,
    confirmDeleteWithUnsavedChanges,
    cancelDeleteWithUnsavedChanges,
    isDeleting: deleteDraftMutation.isPending
  };
}
