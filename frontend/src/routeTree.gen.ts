/* prettier-ignore-start */

/* eslint-disable */

// @ts-nocheck

// noinspection JSUnusedGlobalSymbols

// This file is auto-generated by TanStack Router

// Import Routes

import { Route as rootRoute } from './routes/__root'
import { Route as SourcesImport } from './routes/sources'
import { Route as ActivityImport } from './routes/activity'
import { Route as IndexImport } from './routes/index'

// Create/Update Routes

const SourcesRoute = SourcesImport.update({
  path: '/sources',
  getParentRoute: () => rootRoute,
} as any)

const ActivityRoute = ActivityImport.update({
  path: '/activity',
  getParentRoute: () => rootRoute,
} as any)

const IndexRoute = IndexImport.update({
  path: '/',
  getParentRoute: () => rootRoute,
} as any)

// Populate the FileRoutesByPath interface

declare module '@tanstack/react-router' {
  interface FileRoutesByPath {
    '/': {
      preLoaderRoute: typeof IndexImport
      parentRoute: typeof rootRoute
    }
    '/activity': {
      preLoaderRoute: typeof ActivityImport
      parentRoute: typeof rootRoute
    }
    '/sources': {
      preLoaderRoute: typeof SourcesImport
      parentRoute: typeof rootRoute
    }
  }
}

// Create and export the route tree

export const routeTree = rootRoute.addChildren([
  IndexRoute,
  ActivityRoute,
  SourcesRoute,
])

/* prettier-ignore-end */