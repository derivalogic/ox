
import React, { useState, useEffect } from 'react';
import { SidebarTrigger } from "@/components/ui/sidebar";

export function TopBar() {
  const [currentDateTime, setCurrentDateTime] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentDateTime(new Date());
    }, 1000);

    return () => clearInterval(timer);
  }, []);

  const formatDate = (date: Date) => {
    return date.toISOString().split('T')[0];
  };

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('en-US', { 
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  };

  return (
    <div className="p-4 border-b border-gray-200 bg-primary flex items-center justify-between">
      <SidebarTrigger className="text-white hover:bg-primary/90" />
      <div className="text-white text-sm font-medium">
        {formatDate(currentDateTime)} | {formatTime(currentDateTime)}
      </div>
    </div>
  );
}
