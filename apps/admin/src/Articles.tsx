import { ChangeEvent, useEffect, useState } from "react";
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
  AlertCircle,
  Hourglass,
  CircleCheckBig,
  OctagonX,
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

import ReactQuill from "react-quill-new";
import "react-quill/dist/quill.snow.css";

import { Input } from "./components/ui/input";

import { getBlog } from "./api/blog";
import {
  ListArticlesParams,
  ListingArticleResponseItemsItem,
} from "./api/blog.schemas";
import { SkeletonCard } from "./components/ui/skeleton-card";
import { Alert, AlertDescription, AlertTitle } from "./components/ui/alert";
import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination";
import { Textarea } from "./components/ui/textarea";

const {
  listArticles,
  getArticle,
  deleteArticle,
  createArticle,
  updateArticle,
  publishArticle,
  moveArticleToDraft,
  moveArticleToTrash,
} = getBlog();

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

function article_status(status: string): Status {
  status = status.toLowerCase();

  if (status == "trash") {
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

enum ActionState {
  None = "",
  Active = "active",
  Success = "success",
  Error = "error",
}

function ActionStateMessage({
  action,
  state,
  error,
}: {
  action: string;
  state: string;
  error: string;
}) {
  let color = "";

  switch (state) {
    case ActionState.Active:
      color = "text-blue-500 border-blue-500";
      break;
    case ActionState.Success:
      color = "text-green-500 border-green-500";
      break;
    case ActionState.Error:
      color = "text-red-500 border-red-500";
      break;
  }

  return (
    <div className={`flex flex-row gap-2 rounded-md p-2 text-sm ${color}`}>
      {state === ActionState.Active && <Hourglass size={15} />}
      {state === ActionState.Success && <CircleCheckBig size={15} />}
      {state === ActionState.Error && <OctagonX size={15} />}
      {action}{" "}
      {state === ActionState.Active && (
        <span className="text-blue-500">...</span>
      )}
      {state === ActionState.Error && error && (
        <span className="text-red-500"> - {error}</span>
      )}
    </div>
  );
}

type ChangeStatusType =
    | typeof publishArticle
    | typeof moveArticleToDraft
    | typeof moveArticleToTrash
    | typeof deleteArticle;

export function ArticlesList() {
  const [articles, setArticles] = useState<ListingArticleResponseItemsItem[]>(
    []
  );
  const [viewmode, setViewmode] = useState<ViewMode>(ViewMode.Grid);
  const [status, setStatus] = useState<Status>(Status.Unknown);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [hasSelection, setHasSelection] = useState<boolean>(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState<boolean>(false);
  const [isLoading, setLoading] = useState<boolean>(true);
  const [pages, setPages] = useState<number>(0);
  const [page, setPage] = useState<number>(1);
  const [error, setError] = useState<string>("");
  const [action, setAction] = useState<string>("");
  const [actionError, setActionError] = useState<string>("");
  const [actionState, setActionState] = useState<ActionState>(ActionState.None);

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

  const loadArticles = () => {
    const params: ListArticlesParams = { page };

    if (status !== Status.Unknown) {
      params.status = status;
    }

    setLoading(true);

    setTimeout(() => {
      listArticles(params)
        .then((rs) => {
          const {
            data: { items, pages },
          } = rs;

          setArticles(items);
          setPages(pages);
          setError("");
        })
        .catch((err) => {
          console.error({ err });
        })
        .finally(() => setLoading(false));
    }, 500);
  };

  const statusAction = (msg: string, action: ChangeStatusType) => () => {
    if (selected.size === 0) {
      return;
    }

    setAction(msg.replace("COUNT", selected.size.toString()));
    setActionState(ActionState.Active);

    Promise.all(Array.from(selected).map((id) => action(id)))
      .then(() => {
        setActionState(ActionState.Success);
        loadArticles();
      })
      .catch((err) => {
        setActionState(ActionState.Error);
        setActionError(err.message);
      });
  };

  const publishAction = statusAction(
    "Publish COUNT article(s)",
    publishArticle
  );

  const draftAction = statusAction(
    "Set COUNT article(s) to draft",
    moveArticleToDraft
  );

  const trashAction = statusAction(
    "Move COUNT article(s) to trash",
    moveArticleToTrash
  );

  const deleteAction = statusAction(
    "Permanently deleting COUNT article(s)",
    deleteArticle
  );

  const showDeleteDialog = () => setDeleteDialogOpen(true);
  const hideDeleteDialog = () => setDeleteDialogOpen(false);
  const deleteDialogAction = () => {
    setDeleteDialogOpen(false);
    deleteAction();
  };

  useEffect(loadArticles, [status, page]);

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
          {actionState !== ActionState.Active && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" hidden={!hasSelection}>
                  Change Status
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent>
                <DropdownMenuSeparator />
                <DropdownMenuItem onSelect={publishAction}>
                  <BookOpenText color="green" />
                  Published
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={draftAction}>
                  <NotebookPen />
                  Draft
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={trashAction}>
                  <Trash2 color="red" />
                  Trash
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={showDeleteDialog}>
                  <Trash2 color="red" />
                  Delete Permanently
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          )}
          <ActionStateMessage
            action={action}
            state={actionState}
            error={actionError}
          />
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
              <AlertDialogAction onMouseUp={deleteDialogAction}>
                Continue
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>
      {!isLoading && error === "" && articles.length !== 0 && (
        <Pages pages={pages} page={page} setPage={setPage} />
      )}
      {isLoading && <Loader />}
      {!isLoading && error !== "" && <Alert>Error! {error}</Alert>}
      {!isLoading && error === "" && articles.length === 0 && (
        <Alert>
          <AlertCircle />
          <AlertTitle>No articles found</AlertTitle>
          <AlertDescription>
            No articles found. Try a different filter or add some articles.
          </AlertDescription>
        </Alert>
      )}
      {!isLoading &&
        error === "" &&
        articles.length != 0 &&
        viewmode === ViewMode.Grid && (
          <GridView
            articles={articles}
            selected={selected}
            toggleItem={toggleItem}
          />
        )}
      {!isLoading &&
        error === "" &&
        articles.length != 0 &&
        viewmode === ViewMode.List && (
          <ListView
            articles={articles}
            selected={selected}
            setSelection={setSelected}
          />
        )}
    </div>
  );
}

function Pages({
  pages,
  page,
  setPage,
}: {
  pages: number;
  page: number;
  setPage(n: number): void;
}) {
  return (
    <Pagination>
      <PaginationContent>
        <PaginationItem>
          <PaginationPrevious
            href="#"
            onClick={() => setPage(Math.max(1, page - 1))}
          />
        </PaginationItem>
        {Array.from({ length: pages }, (_, i) => (
          <PaginationItem key={i}>
            <PaginationLink
              href="#"
              onClick={() => setPage(i + 1)}
              className={page === i + 1 ? "active" : ""}
            >
              {i + 1}
            </PaginationLink>
          </PaginationItem>
        ))}
        <PaginationItem>
          <PaginationNext
            href="#"
            onClick={() => setPage(Math.min(pages, page + 1))}
          />
        </PaginationItem>
      </PaginationContent>
    </Pagination>
  );
}

function Loader() {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
      <SkeletonCard />
      <SkeletonCard />
      <SkeletonCard />
      <SkeletonCard />
    </div>
  );
}

function GridView({
  articles,
  selected,
  toggleItem,
}: {
  articles: ListingArticleResponseItemsItem[];
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
  article: ListingArticleResponseItemsItem;
  selected: boolean;
  toggleItem: (id: string, select: boolean) => void;
}) {
  const editLink = `/articles/${article.id}`;
  const status = article_status(article.status);
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

const columns: ColumnDef<ListingArticleResponseItemsItem>[] = [
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
      const s = article_status(status);

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
    cell: ({ row }) => {
      const title = row.getValue("title") as string;
      const id = row.getValue("id") as string;
      return (
        <Link
          to={`/articles/${id}`}
          className="color-blue-200 border-b-2 border-blue-200"
        >
          <span>{title}</span>
        </Link>
      );
    },
  },
  {
    accessorKey: "description",
    header: "Description",
  },
  {
    accessorKey: "updated_at",
    header: "Updated",
    cell: ({ row }) => {
      const t = row.getValue("updated_at") as string;
      return <span>{formatDate(t)}</span>;
    },
  },
  {
    accessorKey: "created_at",
    header: "Created",
    cell: ({ row }) => {
      const t = row.getValue("created_at") as string;
      return <span>{formatDate(t)}</span>;
    },
  },
];

function ListView({
  articles,
  selected,
  setSelection,
}: {
  articles: ListingArticleResponseItemsItem[];
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
  const [title, setTitle] = useState<string>("");
  const [content, setContent] = useState<string>("");
  const [description, setDescription] = useState<string>("");
  const [status, setStatus] = useState<Status>(Status.Unknown);
  const [error, setError] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(true);
  const [action, setAction] = useState<string>("");
  const [actionError, setActionError] = useState<string>("");
  const [actionState, setActionState] = useState<ActionState>(ActionState.None);

  useEffect(() => {
    if (!id) {
      return;
    }

    setLoading(true);

    getArticle(id)
      .then((rs) => {
        const { title, content, description, status } = rs.data;

        setTitle(title);
        setContent(content);
        setDescription(description);
        setStatus(article_status(status));

        setError("");
      })
      .catch((err) => {
        setError(err.message);
      })
      .finally(() => {
        setLoading(false);
      });
  }, [id]);

  const changeTitle = (event: ChangeEvent<HTMLInputElement>) => {
    const { value } = event.target;
    setTitle(value);
  };

  const changeDescription = (event: ChangeEvent<HTMLTextAreaElement>) => {
    const { value } = event.target;
    setDescription(value);
  };

  const onContentChange = (content: string) => {
    setContent(content);
  };

  const saveAction = () => {
    if (!id) {
      return;
    }

    setActionState(ActionState.Active);
    setAction("Saving");

    updateArticle(id, {title, content, description})
      .then(() => {
        setActionState(ActionState.Success);
      }).catch((err) => {
        setActionState(ActionState.Error);
        setActionError(err.message);
      });
  };

  const statusAction = (status: Status, action: ChangeStatusType) => () => {
    if (!id) {
      return;
    }

    setAction("Change Status");
    setActionState(ActionState.Active);

    action(id)
      .then(() => {
        setActionState(ActionState.Success);
        setStatus(status)
      })
      .catch((err) => {
        setActionState(ActionState.Error);
        setActionError(err.message);
      });
  };

  const publishAction = statusAction(Status.Published, publishArticle);

  const draftAction = statusAction(Status.Draft, moveArticleToDraft);

  const trashAction = statusAction(Status.Trash, moveArticleToTrash);

  if (loading) {
    return <SkeletonCard />;
  }

  if (error !== "") {
    return <Alert>Error: {error}</Alert>;
  }

  return (
    <div>
      <div className="flex flex-row justify-between">
        <h1 className="border-b-2">
          <Input
            type="text"
            placeholder="Enter title"
            className="border-none font-bold text-3xl text-gray-500 "
            value={title}
            onChange={changeTitle}
          />
        </h1>
        <div className="flex flex-row justify-end gap-x-5 cursor-pointer">
        <ActionStateMessage action={action} state={actionState} error={actionError} />
          <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline">
                  {status === Status.Published && (<><BookOpenCheck color="green" /><span className="text-green" >Published</span></>)}
                  {status === Status.Draft && (<><NotebookPen /><span>Draft</span></>)}  
                  {status === Status.Trash && (<><Trash2 color="red" /><span className="text-red" >Trash</span></>)}
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent>
                <DropdownMenuSeparator />
                <DropdownMenuItem onSelect={publishAction}>
                  <BookOpenText color="green" />
                  Published
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={draftAction}>
                  <NotebookPen />
                  Draft
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={trashAction}>
                  <Trash2 color="red" />
                  Trash
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
            <Button variant="outline" onMouseUp={saveAction} className="bg-green-600 text-white">
            <Save /> Save
          </Button>
        </div>
      </div>
      <div className="mt-4 flex flex-col gap-2">
        <Label htmlFor="article-description">Description</Label>
        <Textarea placeholder="description" id="article-description" value={description} onChange={changeDescription}></Textarea>
      </div>
      <div className="mt-4">
        <Editor onChange={onContentChange} content={content} />
      </div>
    </div>
  );
}

function Editor({
  content,
  onChange,
}: {
  content: string;
  onChange: (v: string) => void;
}) {
  const [value, setValue] = useState<string>(content);

  const onChanged = (v: string) => {
    setValue(v);
    onChange(v);
  };

  return <ReactQuill value={value} onChange={onChanged} />;
}
