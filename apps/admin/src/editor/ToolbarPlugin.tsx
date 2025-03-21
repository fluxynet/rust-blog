import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { mergeRegister } from "@lexical/utils";
import {
  $getSelection,
  $isRangeSelection,
  CAN_REDO_COMMAND,
  CAN_UNDO_COMMAND,
  FORMAT_ELEMENT_COMMAND,
  FORMAT_TEXT_COMMAND,
  REDO_COMMAND,
  SELECTION_CHANGE_COMMAND,
  UNDO_COMMAND,
} from "lexical";
import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Bold,
  Italic,
  Underline,
  Strikethrough,
  Undo,
  Redo,
  AlignLeft,
  AlignCenter,
  AlignJustify,
  AlignRight,
} from "lucide-react";

const LowPriority = 1;

function ToolbarButton({
  label,
  icon: Icon,
  disabled = false,
  active = false,
  cmd,
}: {
  label: string;
  icon: React.ComponentType;
  disabled?: boolean;
  active?: boolean;
  cmd: () => boolean;
}) {
  let className = "";

  if (disabled) {
    className = "bg-gray-200";
  } else if (active) {
    className = "bg-black text-white";
  }

  return (
    <Button
      variant="outline"
      size="icon"
      disabled={disabled}
      onClick={cmd}
      aria-label={label}
      title={label}
      className={`cursor-pointer ${className}`}
    >
      <Icon/>
    </Button>
  );
}

export default function ToolbarPlugin() {
  const [editor] = useLexicalComposerContext();
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);
  const [isBold, setIsBold] = useState(false);
  const [isItalic, setIsItalic] = useState(false);
  const [isUnderline, setIsUnderline] = useState(false);
  const [isStrikethrough, setIsStrikethrough] = useState(false);

  const $updateToolbar = useCallback(() => {
    const selection = $getSelection();
    if ($isRangeSelection(selection)) {
      setIsBold(selection.hasFormat("bold"));
      setIsItalic(selection.hasFormat("italic"));
      setIsUnderline(selection.hasFormat("underline"));
      setIsStrikethrough(selection.hasFormat("strikethrough"));
    }
  }, []);

  useEffect(() => {
    return mergeRegister(
      editor.registerUpdateListener(({ editorState }) => {
        editorState.read(() => {
          $updateToolbar();
        });
      }),
      editor.registerCommand(
        SELECTION_CHANGE_COMMAND,
        () => {
          $updateToolbar();
          return false;
        },
        LowPriority
      ),
      editor.registerCommand(
        CAN_UNDO_COMMAND,
        (payload) => {
          setCanUndo(payload);
          return false;
        },
        LowPriority
      ),
      editor.registerCommand(
        CAN_REDO_COMMAND,
        (payload) => {
          setCanRedo(payload);
          return false;
        },
        LowPriority
      )
    );
  }, [editor, $updateToolbar]);

  const cmdUndo = () => editor.dispatchCommand(UNDO_COMMAND, undefined);
  const cmdRedo = () => editor.dispatchCommand(REDO_COMMAND, undefined);
  const cmdBold = () => editor.dispatchCommand(FORMAT_TEXT_COMMAND, "bold");
  const cmdItalic = () => editor.dispatchCommand(FORMAT_TEXT_COMMAND, "italic");
  const cmdUnderline = () =>
    editor.dispatchCommand(FORMAT_TEXT_COMMAND, "underline");
  const cmdStrikethrough = () =>
    editor.dispatchCommand(FORMAT_TEXT_COMMAND, "strikethrough");
  const cmdLeft = () => editor.dispatchCommand(FORMAT_ELEMENT_COMMAND, "left");
  const cmdCenter = () =>
    editor.dispatchCommand(FORMAT_ELEMENT_COMMAND, "center");
  const cmdRight = () =>
    editor.dispatchCommand(FORMAT_ELEMENT_COMMAND, "right");
  const cmdJustify = () =>
    editor.dispatchCommand(FORMAT_ELEMENT_COMMAND, "justify");

  return (
    <div className="flex flex-row gap-x-1">
      <ToolbarButton
        label="Undo"
        disabled={canUndo}
        icon={Undo}
        cmd={cmdUndo}
      />
      <ToolbarButton
        label="Redo"
        disabled={canRedo}
        icon={Redo}
        cmd={cmdRedo}
      />
      <ToolbarButton label="Bold" active={isBold} icon={Bold} cmd={cmdBold} />
      <ToolbarButton
        label="Italic"
        active={isItalic}
        icon={Italic}
        cmd={cmdItalic}
      />
      <ToolbarButton
        label="Underline"
        active={isUnderline}
        icon={Underline}
        cmd={cmdUnderline}
      />
      <ToolbarButton
        label="Strikethrough"
        active={isStrikethrough}
        icon={Strikethrough}
        cmd={cmdStrikethrough}
      />
      <ToolbarButton label="Left" icon={AlignLeft} cmd={cmdLeft} />
      <ToolbarButton label="Center" icon={AlignCenter} cmd={cmdCenter} />
      <ToolbarButton label="Right" icon={AlignRight} cmd={cmdRight} />
      <ToolbarButton label="Justify" icon={AlignJustify} cmd={cmdJustify} />
    </div>
  );
}
