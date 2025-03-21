import { useEffect, useState, useRef } from "react";
import { Link, useParams } from "react-router";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  BookOpenCheck,
  BookOpenText,
  Grid,
  List,
  NotebookPen,
  Save,
  Trash2,
  Pencil,
} from "lucide-react";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ColumnDef, RowSelectionState } from "@tanstack/react-table";
import { DataTable } from "./components/ui/data-table";
import { Button } from "./components/ui/button";
import { Switch } from "./components/ui/switch";
import { Label } from "./components/ui/label";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Checkbox } from "./components/ui/checkbox";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

import { formatDate } from "./lib/date";

import { LexicalComposer } from "@lexical/react/LexicalComposer";
import { RichTextPlugin } from "@lexical/react/LexicalRichTextPlugin";
import { ContentEditable } from "@lexical/react/LexicalContentEditable";
import { LexicalErrorBoundary } from "@lexical/react/LexicalErrorBoundary";
import ToolbarPlugin from "./editor/ToolbarPlugin";

import { fakeArticles } from "./fake/articles";

type Article = {
  id: string;
  title: string;
  description: string;
  content: string;
  updated_at: Date;
  created_at: Date;
  deleted_at?: Date;
  status: "published" | "draft";
  author: string;
};

enum ViewMode {
  Grid = "grid",
  List = "list",
}

enum Status {
  Unknown = "unknown",
  Published = "published",
  Draft = "draft",
  Trash = "trash",
}

function article_status(status: string, deleted_at: unknown): Status {
  if (deleted_at) {
    return Status.Trash;
  }

  if (status === "published") {
    return Status.Published;
  }

  if (status === "draft") {
    return Status.Draft;
  }

  return Status.Unknown;
}

export function ArticlesList() {
  const [articles, setArticles] = useState<Article[]>([]);
  const [viewmode, setViewmode] = useState<ViewMode>(ViewMode.Grid);
  const [status, setStatus] = useState<Status>(Status.Unknown);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [hasSelection, setHasSelection] = useState<boolean>(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState<boolean>(false);

  const changeViewMode = (v: string) => setViewmode(v as ViewMode);
  const changeStatus = (v: string) => setStatus(v as Status);
  const toggleItem = (id: string, select: boolean) => {
    if (select) {
      selected.add(id);
    } else {
      selected.delete(id);
    }

    setSelected(selected);
    setHasSelection(selected.size !== 0);
  };

  const showDeleteDialog = () => setDeleteDialogOpen(true);
  const hideDeleteDialog = () => setDeleteDialogOpen(false);
  const deleteDialogAction = () => {
    setDeleteDialogOpen(false);
    console.log({selected, action: "delete_articles"})
  };

  useEffect(() => {
    switch (status) {
      case Status.Unknown:
        setArticles(fakeArticles);
        break;
      case Status.Published:
        setArticles(
          fakeArticles.filter((a) => a.status === "published" && !a.deleted_at)
        );
        break;
      case Status.Draft:
        setArticles(
          fakeArticles.filter((a) => a.status === "draft" && !a.deleted_at)
        );
        break;
      case Status.Trash:
        setArticles(fakeArticles.filter((a) => a.deleted_at));
        break;
    }
  }, [status]);

  return (
    <div className="flex flex-col gap-y-4">
      <div className="flex flex-row justify-between">
        <div className="flex flex-row gap-2">
          <Tabs defaultValue="unknown" onValueChange={changeStatus}>
            <TabsList>
              <TabsTrigger value="unknown">
                <BookOpenText /> All
              </TabsTrigger>
              <TabsTrigger value="published" className="text-green-900">
                <BookOpenCheck color="green" /> Published
              </TabsTrigger>
              <TabsTrigger value="draft">
                <NotebookPen /> Draft
              </TabsTrigger>
              <TabsTrigger value="trash" className="text-red-500">
                <Trash2 color="red" /> Trash
              </TabsTrigger>
            </TabsList>
          </Tabs>
          <DropdownMenu>
            <DropdownMenuTrigger>
              <Button variant="ghost" hidden={!hasSelection}>
                Change Status
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              <DropdownMenuSeparator />
              <DropdownMenuItem>
                <BookOpenText color="green" />
                Published
              </DropdownMenuItem>
              <DropdownMenuItem>
                <NotebookPen />
                Draft
              </DropdownMenuItem>
              <DropdownMenuItem onSelect={showDeleteDialog}>
                <Trash2 color="red" />
                Trash
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
        <Tabs defaultValue="grid" onValueChange={changeViewMode}>
          <div className="flex flex-col items-end">
            <TabsList>
              <TabsTrigger value="grid">
                <Grid />
              </TabsTrigger>
              <TabsTrigger value="list">
                <List />
              </TabsTrigger>
            </TabsList>
          </div>
        </Tabs>
        <AlertDialog open={deleteDialogOpen} onOpenChange={hideDeleteDialog}>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
              <AlertDialogDescription>
                This action cannot be undone. This will permanently delete your
                account and remove your data from our servers.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>Cancel</AlertDialogCancel>
              <AlertDialogAction onMouseUp={deleteDialogAction}>Continue</AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>
      {viewmode === ViewMode.Grid && (
        <GridView
          articles={articles}
          selected={selected}
          toggleItem={toggleItem}
        />
      )}
      {viewmode === ViewMode.List && (
        <ListView
          articles={articles}
          selected={selected}
          setSelection={setSelected}
        />
      )}
    </div>
  );
}

function GridView({
  articles,
  selected,
  toggleItem,
}: {
  articles: Article[];
  selected: Set<string>;
  toggleItem: (id: string, select: boolean) => void;
}) {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
      {articles.map((a) => (
        <GridItem
          key={a.id}
          article={a}
          selected={selected.has(a.id)}
          toggleItem={toggleItem}
        />
      ))}
    </div>
  );
}

function GridItem({
  article,
  selected,
  toggleItem,
}: {
  article: Article;
  selected: boolean;
  toggleItem: (id: string, select: boolean) => void;
}) {
  const editLink = `/articles/${article.id}`;
  const status = article_status(article.status, article.deleted_at);
  const [checked, setChecked] = useState<boolean>(selected);

  const onChecked = (checked: boolean) => {
    setChecked(checked);
    toggleItem(article.id, checked);
  };

  let className = "";

  switch (status) {
    case Status.Published:
      className = "bg-green-100";
      break;
    case Status.Draft:
      className = "bg-gray-100";
      break;
    case Status.Trash:
      className = "bg-red-100";
      break;
  }

  return (
    <Card className={`hover:shadow-gray-500 ${className}`}>
      <CardHeader>
        <CardTitle className="flex justify-between">
          <div>{article.title}</div>
          <div>
            <Switch
              id={`article-grid-${article.id}`}
              checked={checked}
              onCheckedChange={onChecked}
            />
          </div>
        </CardTitle>
        <CardDescription>
          <span title={article.created_at.toLocaleString()}>
            {formatDate(article.created_at)}
          </span>{" "}
          - {article.author}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p>{article.description}</p>
      </CardContent>
      <CardFooter className="flex justify-end">
        <Link to={editLink}>
          <Button variant="outline">
            <Pencil />
          </Button>
        </Link>
      </CardFooter>
    </Card>
  );
}

const columns: ColumnDef<Article>[] = [
  {
    accessorKey: "id",
    header: ({ table }) => (
      <Checkbox
        checked={
          table.getIsAllPageRowsSelected() ||
          (table.getIsSomePageRowsSelected() && "indeterminate")
        }
        onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
        aria-label="Select all"
      />
    ),
    cell: ({ row }) => (
      <Checkbox
        checked={row.getIsSelected()}
        onCheckedChange={(value) => row.toggleSelected(!!value)}
        aria-label="Select row"
      />
    ),
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => {
      const status = row.getValue("status") as string;
      const deleted_at = row.getValue("deleted_at") as Date;
      const s = article_status(status, deleted_at);

      switch (s) {
        case Status.Published:
          return <BookOpenCheck color="green" size="18" />;
        case Status.Draft:
          return <NotebookPen size="18" />;
        case Status.Trash:
          return <Trash2 color="red" size="18" />;
      }

      return <div></div>;
    },
  },
  {
    accessorKey: "author",
    header: "Author",
  },
  {
    accessorKey: "title",
    header: "Title",
  },
  {
    accessorKey: "description",
    header: "Description",
  },
  {
    accessorKey: "updated_at",
    header: "Updated",
    cell: ({ row }) => {
      const t = row.getValue("updated_at") as Date;
      return <span>{formatDate(t)}</span>;
    },
  },
  {
    accessorKey: "created_at",
    header: "Created",
    cell: ({ row }) => {
      const t = row.getValue("created_at") as Date;
      return <span>{formatDate(t)}</span>;
    },
  },
  {
    accessorKey: "deleted_at",
    header: "Deleted",
    cell: ({ row }) => {
      const t = row.getValue("deleted_at") as Date;
      return t ? <span>{formatDate(t)}</span> : "";
    },
  },
];

function ListView({
  articles,
  selected,
  setSelection,
}: {
  articles: Article[];
  selected: Set<string>;
  setSelection: (selection: Set<string>) => void;
}) {
  const onSelectedChange = (selection: RowSelectionState) => {
    const items = new Set<string>(
      Object.keys(selection).filter((key) => selection[key])
    );
    setSelection(items);
  };

  const selection: RowSelectionState = {};
  selected.forEach((id) => (selection[id] = true));

  return (
    <DataTable
      columns={columns}
      data={articles}
      selected={selection}
      onSelectChange={onSelectedChange}
    />
  );
}

export function ArticleEdit() {
  const { id } = useParams<{ id: string }>();
  const [article, setArticle] = useState<Article | null>(null);

  useEffect(() => {
    const a = fakeArticles.find((article) => article.id === id);
    if (a) {
      setArticle(a);
    }
  }, [id]);

  if (!article) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <div className="flex flex-row justify-between">
        <h1
          className="font-bold text-3xl text-gray-500 border-b-2"
          contentEditable={true}
        >
          {article.title}
        </h1>
        <div className="flex flex-row justify-end gap-x-5 cursor-pointer">
          <div className="flex items-center space-x-2">
            <Switch id="article-published" />
            <Label htmlFor="article-published">Publish</Label>
          </div>
          <Button variant="outline" className="bg-green-600 text-white">
            <Save /> Save
          </Button>
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="destructive">
                <Trash2 color="white" /> Delete
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>
                  Are you sure you want to delete this article?
                </AlertDialogTitle>
                <AlertDialogDescription>
                  The article will be unpublished and moved to trash. You can
                  restore this later if you want.
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction>Delete</AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </div>
      <div className="mt-4">
        <Editor />
      </div>
    </div>
  );
}

function onError(error: unknown) {
  console.error(error);
}

function Editor() {
  const initialConfig = {
    namespace: "ArticleEditor",
    //   theme,
    onError,
  };

  return (
    <LexicalComposer initialConfig={initialConfig}>
      <Card>
        <CardHeader>
          <CardDescription>
            <ToolbarPlugin />
          </CardDescription>
        </CardHeader>
        <CardContent>
          <RichTextPlugin
            contentEditable={
              <ContentEditable
                className="min-h-lvh border-2 border-gray-200"
                aria-placeholder="enter some text"
                placeholder={<br />}
              />
            }
            ErrorBoundary={LexicalErrorBoundary}
          />
        </CardContent>
      </Card>
    </LexicalComposer>
  );
}
