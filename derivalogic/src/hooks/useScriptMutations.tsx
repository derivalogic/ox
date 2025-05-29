
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

export function useScriptMutations(script: Script | null) {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const { user } = useAuth();

  // Save script mutation
  const saveScriptMutation = useMutation({
    mutationFn: async ({ scriptName, transactionDetails }: { scriptName: string; transactionDetails: string }) => {
      if (!user?.id || !script?.id) throw new Error('User not authenticated or script not found');
      
      console.log('Updating script with name:', scriptName);
      
      const { data, error } = await supabase
        .from('scripts')
        .update({
          name: scriptName,
          status: 'SAVED'
        })
        .eq('id', script.id)
        .eq('user_id', user.id)
        .select()
        .single();

      if (error) throw error;
      
      console.log('Script updated successfully:', data);
      return { scriptName: data.name, scriptId: data.id };
    },
    onSuccess: ({ scriptName }) => {
      queryClient.invalidateQueries({ queryKey: ['scripts'] });
      queryClient.invalidateQueries({ queryKey: ['events'] });
      toast({ title: "Script saved successfully" });
      console.log('Script saved successfully with name:', scriptName);
    },
    onError: (error) => {
      console.error('Error saving script:', error);
      toast({ title: "Error saving script", variant: "destructive" });
    }
  });

  // Create script mutation for new script
  const createScriptMutation = useMutation({
    mutationFn: async () => {
      if (!user?.id) throw new Error('User not authenticated');
      
      const { data, error } = await supabase
        .from('scripts')
        .insert({
          name: 'New Script*',
          reference_date: new Date().toISOString(),
          status: 'DRAFT',
          user_id: user.id
        })
        .select()
        .single();

      if (error) throw error;
      return data;
    },
    onSuccess: (newScript) => {
      queryClient.invalidateQueries({ queryKey: ['scripts'] });
      toast({ 
        title: "Script created successfully",
        description: `New script is ready for editing`
      });
      return newScript;
    },
    onError: (error) => {
      console.error('Error creating script:', error);
      toast({ title: "Error creating script", variant: "destructive" });
    }
  });

  // Delete draft script mutation
  const deleteDraftScriptMutation = useMutation({
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
      // Don't show toast for silent deletion
    },
    onError: (error) => {
      console.error('Error deleting draft script:', error);
      // Don't show error toast for silent deletion
    }
  });

  return {
    saveScriptMutation,
    createScriptMutation,
    deleteDraftScriptMutation
  };
}
