import type { FC } from 'react';
import type { SerializedEditorState } from 'lexical';

import { AutoFocusPlugin } from '@lexical/react/LexicalAutoFocusPlugin';
import type { InitialConfigType } from '@lexical/react/LexicalComposer';
import { LexicalComposer } from '@lexical/react/LexicalComposer';
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin';
import { ContentEditable } from '@lexical/react/LexicalContentEditable';
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin';
import { LexicalErrorBoundary } from '@lexical/react/LexicalErrorBoundary';
import { OnChangePlugin } from '@lexical/react/LexicalOnChangePlugin';
import { MarkdownShortcutPlugin } from '@lexical/react/LexicalMarkdownShortcutPlugin';
import { HorizontalRulePlugin } from '@lexical/react/LexicalHorizontalRulePlugin';

import { clsx } from 'clsx';
import { EDITOR_NODES, theme } from './settings';

type EditorProps<T> = {
  onChange: (data: T) => void;
  value?: T;
  className?: string;
  editable?: boolean;
  name?: string;
};

export const Editor: FC<EditorProps<SerializedEditorState>> = ({
  onChange,
  value,
  name,
  className,
  editable = true,
}) => {
  const initialConfig: InitialConfigType = {
    namespace: 'MyEditor',
    theme,
    onError: console.error,
    editorState(editor) {
      if (!value) return;
      const state = editor.parseEditorState(value);

      editor.setEditorState(state);
    },
    nodes: EDITOR_NODES,
    editable,
  };

  return (
    <div className={clsx('grid grid-cols-1 grid-rows-1', className)}>
      <LexicalComposer initialConfig={initialConfig}>
        <HorizontalRulePlugin />
        <MarkdownShortcutPlugin />
        <RichTextPlugin
          contentEditable={(
            <ContentEditable
              className="col-start-1 col-end-1 row-start-1 row-end-1 outline-none z-0"
              name={name}
            />
          )}
          placeholder={(
            <div
              className="col-start-1 col-end-1 row-start-1 row-end-1 pointer-events-none font-extrabold text-text-800 -z-10"
            >
              Enter some text...
            </div>
          )}
          ErrorBoundary={LexicalErrorBoundary}
        />
        <HistoryPlugin />
        <AutoFocusPlugin />
        <OnChangePlugin onChange={state => onChange(state.toJSON())} />
      </LexicalComposer>
    </div>
  );
};
