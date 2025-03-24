import { BookOpenText, LogOut } from "lucide-react";
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
import { User, useAuth } from "./services/auth";
import { Avatar, AvatarImage } from "./components/ui/avatar";
import LoginPage from "./Login";

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
      <NavigationMenuLink asChild>
        <Button variant="link" className="flex flex-row">
          <Icon /> {label}
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
          <MenuItem href="/articles" label="Articles" icon={BookOpenText} />
        </NavigationMenuItem>
      </NavigationMenuList>
    </NavigationMenu>
  );
}

function UserWidget({ user }: { user: User }) {
  return (
    <div className="flex flex-row items-center gap-x-4">
      <Avatar>
        <AvatarImage src={user.avatar_url} alt={user.name} />
      </Avatar>
      <NavigationMenu>
        <NavigationMenuList>
          <NavigationMenuItem>
            <NavigationMenuTrigger>{user.name}</NavigationMenuTrigger>
            <NavigationMenuContent>
              <a href="/api/auth/logout">
                <span className="flex flex-row gap-x-1">
                  <LogOut /> Log Out
                </span>
              </a>
            </NavigationMenuContent>
          </NavigationMenuItem>
        </NavigationMenuList>
      </NavigationMenu>
    </div>
  );
}

export default function Layout({ children }: { children: React.ReactNode }) {
  const { user, isLoggedIn } = useAuth();
  return (
    <>
      {!isLoggedIn && <LoginPage />}
      {isLoggedIn && (
        <div className="flex flex-col min-h-screen p-4">
          <header className="flex flex-row gap-x-4 rounded-md bg-gray-100 p-4 shadow-md justify-between">
            <div className="flex flex-row items-center">
              <NavLink to="/">📖 Blog inc.</NavLink>
              <Nav />
            </div>
            <div>{user && <UserWidget user={user} />}</div>
          </header>
          <div className="flex flex-1">
            <main className="flex-1 p-4">{children}</main>
          </div>
        </div>
      )}
    </>
  );
}
