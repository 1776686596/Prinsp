export type ToolType = 'select' | 'arrow' | 'rect' | 'text' | 'mosaic'

export interface Point {
  x: number
  y: number
}

export interface Annotation {
  id: string
  type: ToolType
  points: Point[]
  style: {
    color: string
    lineWidth: number
    fontSize?: number
  }
  text?: string
}

export interface Selection {
  x: number
  y: number
  width: number
  height: number
}