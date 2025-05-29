
import React, { useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Alert, AlertDescription } from "@/components/ui/alert";

interface NewScriptModalProps {
  open: boolean;
  onClose: () => void;
  onAddScript: (scriptName: string) => void;
  existingScriptNames: string[];
}

const NewScriptModal: React.FC<NewScriptModalProps> = ({ 
  open, 
  onClose, 
  onAddScript, 
  existingScriptNames 
}) => {
  const [scriptName, setScriptName] = useState('Untitled Script');
  const [alert, setAlert] = useState<string | null>(null);

  const handleAddScript = () => {
    if (scriptName.trim() === '') {
      setAlert('Script name is required.');
      return;
    }
    if (existingScriptNames.includes(scriptName.trim())) {
      setAlert('Script name must be unique.');
      return;
    }
    onAddScript(scriptName.trim());
    setScriptName('Untitled Script');
    setAlert(null);
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Add New Script</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          {alert && (
            <Alert variant="destructive">
              <AlertDescription>{alert}</AlertDescription>
            </Alert>
          )}
          <div className="space-y-2">
            <Label htmlFor="scriptName">Script Name</Label>
            <Input
              id="scriptName"
              value={scriptName}
              onChange={(e) => setScriptName(e.target.value)}
              placeholder="Untitled Script"
            />
          </div>
          <div className="flex justify-end space-x-2">
            <Button onClick={handleAddScript}>
              Add
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default NewScriptModal;
