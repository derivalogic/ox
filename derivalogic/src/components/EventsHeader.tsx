
import React, { useState, useEffect } from 'react';
import { Button } from "@/components/ui/button";
import { Edit, Pencil } from "lucide-react";

interface EventsHeaderProps {
  currentScriptName: string;
  scriptStatus: string;
  showSavedMessage: boolean;
  isModifyMode: boolean;
  scriptId: string;
  referenceDate: Date;
  eventsCount: number;
  onSave: (scriptName?: string) => void;
  onModify: () => void;
  onScriptNameChange?: (name: string) => void;
}

export function EventsHeader({
  currentScriptName,
  scriptStatus,
  showSavedMessage,
  isModifyMode,
  scriptId,
  referenceDate,
  eventsCount,
  onSave,
  onModify,
  onScriptNameChange
}: EventsHeaderProps) {
  const [scriptName, setScriptName] = useState(currentScriptName);
  const [isEditingName, setIsEditingName] = useState(false);

  // Update local state when currentScriptName changes
  useEffect(() => {
    setScriptName(currentScriptName);
  }, [currentScriptName]);

  const formatDate = (date: Date) => {
    return date.toISOString().split('T')[0];
  };

  const handleScriptNameChange = (value: string) => {
    // Limit to 60 characters
    const limitedValue = value.slice(0, 60);
    setScriptName(limitedValue);
    if (onScriptNameChange) {
      onScriptNameChange(limitedValue);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      setIsEditingName(false);
    }
  };

  const handlePencilClick = () => {
    setIsEditingName(true);
  };

  return (
    <div className="border-b border-gray-200 pb-8 mb-8 bg-white rounded-lg">
      <div className="flex items-center justify-between mb-6">
        <div className="flex flex-col gap-3">
          <label className="text-sm font-semibold text-gray-700 uppercase tracking-wide">Script Name</label>
          <div className="flex items-center gap-3">
            {isModifyMode && isEditingName ? (
              <input
                type="text"
                value={scriptName}
                onChange={(e) => handleScriptNameChange(e.target.value)}
                onKeyDown={handleKeyDown}
                onBlur={() => setIsEditingName(false)}
                className="text-2xl font-bold text-gray-900 border-2 border-blue-300 bg-white px-4 py-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all duration-200 max-w-lg"
                placeholder="Enter script name"
                maxLength={60}
                autoFocus
              />
            ) : (
              <>
                <h1 className="text-3xl font-bold text-gray-900">{scriptName}</h1>
                {isModifyMode && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handlePencilClick}
                    className="h-10 w-10 p-0 hover:bg-gray-100 rounded-lg"
                  >
                    <Pencil className="h-4 w-4 text-gray-500" />
                  </Button>
                )}
              </>
            )}
          </div>
          {scriptName.length >= 60 && isModifyMode && (
            <div className="text-xs text-red-500 font-medium">
              {scriptName.length}/60 characters
            </div>
          )}
          {!isModifyMode && (
            <div className="h-1 w-16 bg-blue-500 rounded-full"></div>
          )}
        </div>
        <div className="flex items-center gap-3">
          {/* Show save/modify buttons for saved scripts */}
          {scriptStatus !== 'DRAFT' && (
            <>
              {isModifyMode ? (
                <Button
                  className="bg-blue-600 hover:bg-blue-700 text-white h-10 px-6 rounded-lg shadow-sm hover:shadow-md transition-all duration-200 font-medium"
                  onClick={() => onSave(scriptName)}
                >
                  Save Changes
                </Button>
              ) : (
                <Button
                  className="bg-blue-600 hover:bg-blue-700 text-white h-10 px-6 rounded-lg shadow-sm hover:shadow-md transition-all duration-200 font-medium flex items-center gap-2"
                  onClick={onModify}
                >
                  <Edit className="h-4 w-4" />
                  Modify Script
                </Button>
              )}
            </>
          )}
        </div>
      </div>
      <div className="flex items-center gap-8 text-sm">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 bg-gray-400 rounded-full"></div>
          <span className="text-gray-600 font-medium">ID:</span>
          <span className="text-gray-800 font-mono bg-gray-100 px-2 py-1 rounded">{scriptId.slice(0, 8)}...</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 bg-blue-400 rounded-full"></div>
          <span className="text-gray-600 font-medium">Created:</span>
          <span className="text-gray-800">{formatDate(referenceDate)}</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 bg-green-400 rounded-full"></div>
          <span className="text-gray-600 font-medium">Events:</span>
          <span className="text-gray-800 font-semibold">{eventsCount}</span>
        </div>
      </div>
    </div>
  );
}
