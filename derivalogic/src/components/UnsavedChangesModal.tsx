
import React from 'react';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

interface UnsavedChangesModalProps {
  open: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  isDeletingDraft?: boolean;
}

export function UnsavedChangesModal({ 
  open, 
  onConfirm, 
  onCancel, 
  isDeletingDraft = false 
}: UnsavedChangesModalProps) {
  return (
    <AlertDialog open={open}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            {isDeletingDraft ? 'Delete Draft with Unsaved Changes' : 'Unsaved Changes'}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {isDeletingDraft 
              ? 'This draft has unsaved changes that will be lost if you delete it. Are you sure you want to permanently delete this draft?'
              : 'You have unsaved changes that will be lost if you leave this page. Draft scripts that haven\'t been saved will be permanently deleted. Are you sure you want to continue?'
            }
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={onCancel}>Cancel</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm} className="bg-red-600 hover:bg-red-700">
            {isDeletingDraft ? 'Delete Draft' : 'Leave Without Saving'}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
