
import React from 'react';
import { FormField, FormItem, FormLabel, FormControl, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Control } from 'react-hook-form';

interface TransactionFormData {
  scriptName: string;
  scriptId: string;
  status: string;
  createdDate: string;
  transactionType: string;
  referenceDate: string;
  eventsCount: string;
  counterpartyName: string;
  contactInformation: string;
  additionalDetails: string;
  notes: string;
  lastModified: string;
}

interface TransactionScriptInfoProps {
  control: Control<TransactionFormData>;
}

export function TransactionScriptInfo({ control }: TransactionScriptInfoProps) {
  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-900 border-b border-gray-200 pb-2">
        Script Information
      </h2>
      <div className="grid gap-4 md:grid-cols-2">
        <FormField
          control={control}
          name="scriptName"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Script Name</FormLabel>
              <FormControl>
                <Input {...field} readOnly className="bg-gray-50" />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={control}
          name="scriptId"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Script ID</FormLabel>
              <FormControl>
                <Input {...field} readOnly className="bg-gray-50 font-mono" />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={control}
          name="status"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Status</FormLabel>
              <FormControl>
                <Input {...field} readOnly className="bg-gray-50" />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={control}
          name="createdDate"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Created Date</FormLabel>
              <FormControl>
                <Input {...field} readOnly className="bg-gray-50" />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
      </div>
    </div>
  );
}
