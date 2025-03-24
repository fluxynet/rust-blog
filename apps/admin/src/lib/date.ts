import { format, formatDistanceToNow, differenceInHours } from "date-fns";

export function formatDate(date: string): string {
  const parsedDate = new Date(date);
  const now = new Date();
  const diffInHours = differenceInHours(now, parsedDate);
  
  if (diffInHours < 24) {
    return formatDistanceToNow(parsedDate, { addSuffix: true });
  }
  
  return format(parsedDate, "MMM d, yyyy hh:mm");
};