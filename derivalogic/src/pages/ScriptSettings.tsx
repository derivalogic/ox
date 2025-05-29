import React, { useEffect, useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { supabase } from '@/integrations/supabase/client';
import { useAuth } from '@/hooks/useAuth';
import { useToast } from '@/hooks/use-toast';
import MainLayout from '@/components/MainLayout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from '@/components/ui/alert-dialog';
import { Settings, Trash2 } from 'lucide-react';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

export default function ScriptSettings() {
  const location = useLocation();
  const navigate = useNavigate();
  const script = location.state?.script as Script;
  const { user } = useAuth();
  const { toast } = useToast();
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!script || !user) {
      navigate('/');
    }
  }, [script, user, navigate]);

  // Fetch all scripts for the sidebar
  const { data: scripts = [] } = useQuery({
    queryKey: ['scripts'],
    queryFn: async () => {
      if (!user?.id) return [];
      
      const { data, error } = await supabase
        .from('scripts')
        .select('*')
        .eq('user_id', user.id)
        .order('created_at', { ascending: false });

      if (error) {
        console.error('Error fetching scripts:', error);
        throw error;
      }

      return data.map(script => ({
        id: script.id,
        name: script.name || 'Untitled',
        referenceDate: new Date(script.reference_date),
        status: script.status,
        events: []
      }));
    },
    enabled: !!user?.id
  });

  // Delete script mutation
  const deleteScriptMutation = useMutation({
    mutationFn: async () => {
      if (!user?.id || !script?.id) throw new Error('User not authenticated or script not found');
      
      // First delete all events associated with the script
      const { error: eventsError } = await supabase
        .from('events')
        .delete()
        .eq('script_id', script.id)
        .eq('user_id', user.id);

      if (eventsError) throw eventsError;

      // Then delete the script
      const { error: scriptError } = await supabase
        .from('scripts')
        .delete()
        .eq('id', script.id)
        .eq('user_id', user.id);

      if (scriptError) throw scriptError;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scripts'] });
      toast({ title: "Script deleted successfully" });
      navigate('/');
    },
    onError: (error) => {
      console.error('Error deleting script:', error);
      toast({ title: "Error deleting script", variant: "destructive" });
    }
  });

  const handleScriptSelect = (selectedScript: Script) => {
    navigate('/script-settings', { state: { script: selectedScript } });
  };

  const handleNewScript = () => {
    navigate('/');
  };

  const handleDeleteScript = () => {
    deleteScriptMutation.mutate();
  };

  if (!script || !user) {
    return <div className="text-white">Loading...</div>;
  }

  return (
    <MainLayout 
      scripts={scripts}
      currentScript={script}
      onScriptSelect={handleScriptSelect}
      onNewScript={handleNewScript}
    >
      <div className="flex items-center gap-2 mb-6">
        <Settings className="h-6 w-6 text-blue-500" />
        <h1 className="text-2xl font-semibold text-gray-900">Script Settings</h1>
      </div>
      <p className="text-gray-600 mb-6">Configure settings for {script.name}</p>
      
      <Card className="border-red-200">
        <CardHeader>
          <CardTitle className="text-red-600">
            Danger Zone
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm text-gray-600">
            Deleting this script will permanently remove all associated events and data. This action cannot be undone.
          </p>
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="destructive" className="flex items-center gap-2">
                <Trash2 className="h-4 w-4" />
                Delete Script
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
                <AlertDialogDescription>
                  This action cannot be undone. This will permanently delete the script "{script.name}" 
                  and all of its associated events.
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction 
                  onClick={handleDeleteScript}
                  className="bg-red-600 hover:bg-red-700"
                  disabled={deleteScriptMutation.isPending}
                >
                  {deleteScriptMutation.isPending ? 'Deleting...' : 'Delete'}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </CardContent>
      </Card>
    </MainLayout>
  );
}
