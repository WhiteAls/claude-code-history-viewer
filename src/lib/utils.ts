import { clsx, type ClassValue } from "clsx"
import { extendTailwindMerge } from "tailwind-merge"

// Custom font-size utilities defined in src/index.css (text-2xs/3xs and the
// reactive text-pxNN scale from issue #408). tailwind-merge doesn't know these
// belong to the `font-size` group, so by default it treats e.g. `text-px12`
// as conflicting with a `text-foreground` color and drops the size — leaving
// the element at the inherited base size. Registering them keeps size + color.
const twMerge = extendTailwindMerge({
  extend: {
    classGroups: {
      "font-size": [
        "text-2xs",
        "text-3xs",
        "text-px6",
        "text-px8",
        "text-px9",
        "text-px10",
        "text-px11",
        "text-px12",
        "text-px13",
        "text-px14",
      ],
    },
  },
})

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
