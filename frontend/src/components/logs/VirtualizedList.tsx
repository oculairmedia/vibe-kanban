import { Virtuoso, VirtuosoHandle } from 'react-virtuoso';
import { useEffect, useMemo, useRef, useState } from 'react';

import DisplayConversationEntry from '../NormalizedConversation/DisplayConversationEntry';
import { useEntries } from '@/contexts/EntriesContext';
import {
  AddEntryType,
  PatchTypeWithKey,
  useConversationHistory,
} from '@/hooks/useConversationHistory';
import { Loader2 } from 'lucide-react';
import { TaskAttempt, TaskWithAttemptStatus } from 'shared/types';
import { ApprovalFormProvider } from '@/contexts/ApprovalFormContext';

interface VirtualizedListProps {
  attempt: TaskAttempt;
  task?: TaskWithAttemptStatus;
}

const VirtualizedList = ({ attempt, task }: VirtualizedListProps) => {
  const [entries, setChannelData] = useState<PatchTypeWithKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [shouldAutoScroll, setShouldAutoScroll] = useState(true);
  const { setEntries, reset } = useEntries();
  const virtuosoRef = useRef<VirtuosoHandle>(null);
  const prevEntriesLengthRef = useRef(0);

  useEffect(() => {
    setLoading(true);
    setChannelData([]);
    reset();
    setShouldAutoScroll(true);
  }, [attempt.id, reset]);

  const onEntriesUpdated = (
    newEntries: PatchTypeWithKey[],
    addType: AddEntryType,
    newLoading: boolean
  ) => {
    setChannelData(newEntries);
    setEntries(newEntries);

    // Auto-scroll to bottom when new entries are added during running state
    if (addType === 'running' && !loading && shouldAutoScroll) {
      // Delay scroll to allow DOM to update
      setTimeout(() => {
        virtuosoRef.current?.scrollToIndex({
          index: newEntries.length - 1,
          behavior: 'smooth',
          align: 'end',
        });
      }, 50);
    }

    // Scroll to bottom when initial data loads
    if (loading && !newLoading && newEntries.length > 0) {
      setTimeout(() => {
        virtuosoRef.current?.scrollToIndex({
          index: newEntries.length - 1,
          behavior: 'auto',
          align: 'end',
        });
      }, 100);
    }

    if (loading) {
      setLoading(newLoading);
    }

    prevEntriesLengthRef.current = newEntries.length;
  };

  useConversationHistory({ attempt, onEntriesUpdated });

  const messageListContext = useMemo(
    () => ({ attempt, task }),
    [attempt, task]
  );

  const itemContent = (_index: number, data: PatchTypeWithKey) => {
    if (data.type === 'STDOUT') {
      return <p>{data.content}</p>;
    }
    if (data.type === 'STDERR') {
      return <p>{data.content}</p>;
    }
    if (data.type === 'NORMALIZED_ENTRY' && messageListContext.attempt) {
      return (
        <DisplayConversationEntry
          expansionKey={data.patchKey}
          entry={data.content}
          executionProcessId={data.executionProcessId}
          taskAttempt={messageListContext.attempt}
          task={messageListContext.task}
        />
      );
    }

    return null;
  };

  return (
    <ApprovalFormProvider>
      <Virtuoso
        ref={virtuosoRef}
        className="flex-1"
        data={entries}
        itemContent={itemContent}
        computeItemKey={(_index, data) => `l-${data.patchKey}`}
        components={{
          Header: () => <div className="h-2"></div>,
          Footer: () => <div className="h-2"></div>,
        }}
        followOutput="smooth"
      />
      {loading && (
        <div className="absolute top-0 left-0 w-full h-full bg-primary flex flex-col gap-2 justify-center items-center">
          <Loader2 className="h-8 w-8 animate-spin" />
          <p>Loading History</p>
        </div>
      )}
    </ApprovalFormProvider>
  );
};

export default VirtualizedList;
