
import React from 'react';
import MainLayout from '@/components/MainLayout';
import { TransactionDetailsContainer } from '@/components/TransactionDetailsContainer';
import { useTransactionDetails } from '@/hooks/useTransactionDetails';

export default function TransactionDetails() {
  const {
    script,
    user,
    scripts,
    form,
    handleScriptSelect,
    handleNewScript,
    onSubmit
  } = useTransactionDetails();

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
      <TransactionDetailsContainer
        script={script}
        scripts={scripts}
        form={form}
        onSubmit={onSubmit}
      />
    </MainLayout>
  );
}
