
import { useCallback } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { supabase } from '@/integrations/supabase/client';
import { useToast } from '@/hooks/use-toast';
import { useAuth } from '@/hooks/useAuth';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

export function useSidebarSave() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const { user } = useAuth();

  const saveScriptFromSidebar = useMutation({
    mutationFn: async ({ scriptId, scriptName }: { scriptId: string; scriptName?: string }) => {
      if (!user?.id) throw new Error('User not authenticated');

      const { data, error } = await supabase
        .from('scripts')
        .update({
          status: 'SAVED',
          ...(scriptName ? { name: scriptName } : {})
        })
        .eq('id', scriptId)
        .eq('user_id', user.id)
        .select()
        .single();

      if (error) throw error;
      return data;
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['scripts'] });
      queryClient.invalidateQueries({ queryKey: ['events'] });
      toast({ 
        title: "Script saved successfully",
        description: `"${data.name}" has been saved.`
      });
    },
    onError: (error) => {
      console.error('Error saving script:', error);
      toast({ 
        title: "Error saving script", 
        description: "Failed to save the script.",
        variant: "destructive" 
      });
    }
  });

  const handleSaveScript = useCallback(
    (script: Script, name?: string) => {
      if (script.status === 'DRAFT') {
        saveScriptFromSidebar.mutate({ scriptId: script.id, scriptName: name ?? script.name });
      }
    },
    [saveScriptFromSidebar]
  );

  return {
    handleSaveScript,
    isSaving: saveScriptFromSidebar.isPending
  };
}
