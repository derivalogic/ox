
import React, { useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { useAuth } from '@/hooks/useAuth';
import { supabase } from '@/integrations/supabase/client';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useToast } from '@/hooks/use-toast';

interface CounterpartyModalProps {
  open: boolean;
  onClose: () => void;
  onSave: (counterpartyId: string) => void;
}

export function CounterpartyModal({ open, onClose, onSave }: CounterpartyModalProps) {
  const [name, setName] = useState('');
  const [contactEmail, setContactEmail] = useState('');
  const [contactPhone, setContactPhone] = useState('');
  const [address, setAddress] = useState('');
  const { user } = useAuth();
  const queryClient = useQueryClient();
  const { toast } = useToast();

  const createCounterpartyMutation = useMutation({
    mutationFn: async () => {
      if (!user?.id) throw new Error('User not authenticated');
      
      const { data, error } = await supabase
        .from('counterparties')
        .insert({
          user_id: user.id,
          name,
          contact_email: contactEmail || null,
          contact_phone: contactPhone || null,
          address: address || null
        })
        .select()
        .single();

      if (error) throw error;
      return data;
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['counterparties'] });
      toast({ title: "Counterparty created successfully" });
      onSave(data.id);
      handleClose();
    },
    onError: (error) => {
      console.error('Error creating counterparty:', error);
      toast({ 
        title: "Error creating counterparty", 
        variant: "destructive" 
      });
    }
  });

  const handleSave = () => {
    if (!name.trim()) {
      toast({ 
        title: "Name is required", 
        variant: "destructive" 
      });
      return;
    }
    createCounterpartyMutation.mutate();
  };

  const handleClose = () => {
    setName('');
    setContactEmail('');
    setContactPhone('');
    setAddress('');
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Add Counterparty</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div>
            <Label htmlFor="counterparty-name">Name *</Label>
            <Input
              id="counterparty-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Counterparty name"
              required
            />
          </div>
          <div>
            <Label htmlFor="counterparty-email">Contact Email</Label>
            <Input
              id="counterparty-email"
              type="email"
              value={contactEmail}
              onChange={(e) => setContactEmail(e.target.value)}
              placeholder="contact@example.com"
            />
          </div>
          <div>
            <Label htmlFor="counterparty-phone">Contact Phone</Label>
            <Input
              id="counterparty-phone"
              value={contactPhone}
              onChange={(e) => setContactPhone(e.target.value)}
              placeholder="+1 (555) 123-4567"
            />
          </div>
          <div>
            <Label htmlFor="counterparty-address">Address</Label>
            <Textarea
              id="counterparty-address"
              value={address}
              onChange={(e) => setAddress(e.target.value)}
              placeholder="Full address"
              rows={3}
            />
          </div>
        </div>
        <div className="flex justify-end gap-2 mt-6">
          <Button variant="outline" onClick={handleClose}>
            Cancel
          </Button>
          <Button 
            onClick={handleSave}
            disabled={createCounterpartyMutation.isPending}
          >
            {createCounterpartyMutation.isPending ? 'Creating...' : 'Save'}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
