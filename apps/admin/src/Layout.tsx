import { Home, BookOpenText, History, Images, LogOut } from "lucide-react";
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
} from "@/components/ui/navigation-menu";
import { NavLink } from "react-router";
import { Button } from "./components/ui/button";

function MenuItem({
  label,
  href,
  icon: Icon,
}: {
  label: string;
  href: string;
  icon: React.ComponentType;
}) {
  return (
    <NavLink to={href}>
      <NavigationMenuLink>
        <Button variant="link">
          <Icon/> {label}
        </Button>
      </NavigationMenuLink>
    </NavLink>
  );
}

function Nav() {
  return (
    <NavigationMenu>
      <NavigationMenuList>
        <NavigationMenuItem>
          <MenuItem href="/" label="Home" icon={Home} />
        </NavigationMenuItem>
        <NavigationMenuItem>
          <MenuItem href="/articles" label="Articles" icon={BookOpenText} />
        </NavigationMenuItem>
        <NavigationMenuItem>
          <MenuItem href="/" label="Images" icon={Images} />
        </NavigationMenuItem>
        <NavigationMenuItem>
          <MenuItem href="/" label="History" icon={History} />
        </NavigationMenuItem>
      </NavigationMenuList>
    </NavigationMenu>
  );
}

function User() {
  return (
    <NavigationMenu>
      <NavigationMenuList>
        <NavigationMenuItem>
          <NavigationMenuTrigger>John Doe</NavigationMenuTrigger>
          <NavigationMenuContent>
            <MenuItem href="/" label="Logout" icon={LogOut} />
          </NavigationMenuContent>
        </NavigationMenuItem>
      </NavigationMenuList>
    </NavigationMenu>
  );
}

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex flex-col min-h-screen p-4">
      <header className="flex flex-row gap-x-4 rounded-md bg-gray-100 p-4 shadow-md justify-between">
        <div className="flex flex-row items-center">
          <NavLink to="/">📖 Blog inc.</NavLink>
          <Nav />
        </div>
        <div>
          <User />
        </div>
      </header>
      <div className="flex flex-1">
        <main className="flex-1 p-4">{children}</main>
      </div>
    </div>
  );
}
