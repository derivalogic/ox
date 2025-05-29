import React, { useState, useEffect } from 'react';
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
import { Alert, AlertDescription } from "@/components/ui/alert";

interface ScriptSaveModalProps {
  open: boolean;
  onClose: () => void;
  onSave: (scriptName: string, counterpartyInfo: string) => void;
  initialName?: string;
}

export function ScriptSaveModal({
  open,
  onClose,
  onSave,
  initialName = ''
}: ScriptSaveModalProps) {
  const [scriptName, setScriptName] = useState('');
  const [counterpartyInfo, setCounterpartyInfo] = useState('');
  const [alert, setAlert] = useState<string | null>(null);

  // Update script name when modal opens or initialName changes
  useEffect(() => {
    if (open) {
      // Only remove the '*' if it exists, otherwise keep the original name
      const cleanName = initialName.endsWith('*') ? initialName.slice(0, -1) : initialName;
      setScriptName(cleanName);
    }
  }, [open, initialName]);

  const handleSave = () => {
    if (scriptName.trim() === '') {
      setAlert('Script name is required.');
      return;
    }
    onSave(scriptName.trim(), counterpartyInfo.trim());
    setAlert(null);
    onClose();
  };

  const handleClose = () => {
    setCounterpartyInfo('');
    setAlert(null);
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Save Script</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          {alert && (
            <Alert variant="destructive">
              <AlertDescription>{alert}</AlertDescription>
            </Alert>
          )}
          <div>
            <Label htmlFor="script-name">Script Name</Label>
            <Input
              id="script-name"
              value={scriptName}
              onChange={(e) => setScriptName(e.target.value)}
              placeholder="Enter script name"
            />
          </div>
          <div>
            <Label htmlFor="counterparty-info">Transaction Details</Label>
            <Textarea
              id="counterparty-info"
              value={counterpartyInfo}
              onChange={(e) => setCounterpartyInfo(e.target.value)}
              placeholder="Enter transaction details, counterparty information, etc."
              rows={4}
            />
          </div>
        </div>
        <div className="flex justify-end gap-2 mt-6">
          <Button variant="outline" onClick={handleClose} className="h-8">
            Cancel
          </Button>
          <Button onClick={handleSave} className="h-8">
            Save
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
