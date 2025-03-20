import { format, formatDistanceToNow, differenceInHours } from "date-fns";

export function formatDate(date: Date): string {
    const now = new Date()
    const diffInHours = differenceInHours(now, date)
  
    if (diffInHours < 24) {
      return formatDistanceToNow(date, { addSuffix: true })
    }
    
      return format(date, "MMM d, yyyy hh:mm")
  };