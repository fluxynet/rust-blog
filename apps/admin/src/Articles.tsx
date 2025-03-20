import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Grid, List } from "lucide-react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ColumnDef } from "@tanstack/react-table";
import { DataTable } from "./components/ui/data-table";
import { formatDate } from "./lib/date";
import { fakeArticles } from "./fake/articles";

type Article = {
  id: string;
  title: string;
  description: string;
  content: string;
  updated_at: Date;
  created_at: Date;
  author: string;
};

export function ArticlesList() {
  return (
    <Tabs defaultValue="grid" className="">
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
      <TabsContent value="grid">
        <GridView />
      </TabsContent>
      <TabsContent value="list">
        <ListView />
      </TabsContent>
    </Tabs>
  );
}

function GridView() {
  return (
    <div className="flex flex-row flex-wrap gap-y-4 gap-2 justify-items-center">
      {fakeArticles.map((a) => (
        <GridItem key={a.id} article={a} />
      ))}
    </div>
  );
}

function GridItem({ article }: { article: Article }) {
  return (
    <div className="w-sm cursor-pointer">
      <Card className="hover:bg-gray-200">
        <CardHeader>
          <CardTitle>{article.title}</CardTitle>
          <CardDescription>
            <span title={article.created_at.toLocaleString()}>{formatDate(article.created_at)}</span> - {article.author}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p>{article.description}</p>
        </CardContent>
      </Card>
    </div>
  );
}

const columns: ColumnDef<Article>[] = [
  {
    accessorKey: "id",
    header: "#",
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
    cell: ({row}) => {
        const t = row.getValue("updated_at") as Date
        return <span>{formatDate(t)}</span>
    },
  },
  {
    accessorKey: "created_at",
    header: "Created",
    cell: ({row}) => {
        const t = row.getValue("updated_at") as Date
        return <span>{formatDate(t)}</span>
    },
  },
];

function ListView() {
  return <DataTable columns={columns} data={fakeArticles} />;
}
