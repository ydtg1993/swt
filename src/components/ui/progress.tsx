import * as React from "react"
import { Root, Indicator } from "@radix-ui/react-progress"

import { cn } from "@/lib/utils"

function Progress({
  className,
  value,
  ...props
}: React.ComponentProps<typeof Root>) {
  return (
    <Root
      data-slot="progress"
      className={cn(
        "relative flex h-4 w-full items-center overflow-hidden rounded border-2 bg-background",
        className
      )}
      {...props}
    >
      <Indicator
        data-slot="progress-indicator"
        className="size-full flex-1 bg-primary transition-all"
        style={{ transform: `translateX(-${100 - (value || 0)}%)` }}
      />
    </Root>
  )
}

export { Progress }
