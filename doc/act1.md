# Act 1

### Tentative Due Date: May 1, 2026.

What needs to be done

- PKM 
  - [ ] Atomic note IDs
  - [ ] Tagging
  - [ ] Filtering
  - [ ] Back-Links
  - [ ] Graph Visualizer
    - Only need to visualize nodes, no need to have a control layer from TUI.

- TODO 
  - [ ] Tasks as Notes
  - [ ] Due Dates / Priorities
  - [ ] Priority List
  - [ ] Groups
  
- Navigation
  - [ ] Fuzzy finder navigation
  - [ ] Vim / helix key bindings


## Plan of Action

Backend server that interfaces with the file-system.
This way it can be accessible from any web browser, (that is authenticated
of course).

Uses gRPC as the communication framework.
  - Efficiency
  - Generated client server implementations.

But now we have to sync the filesystem the entire time since that's
behind the server. Well not necessarily... Since we are typically
writing one by one, it shouldn't be too bad. [g-d] wouldn't work either...

I'm reconsidering this... maybe instead we can have a "host-server" that
maps the thing away.

So for now lets not do a backend server,
instead lets just have everything in the folder.

Going to be build off Emergence.

Lets also keep tasks not completely as files, we keep it mostly in
database, and then link each task to a description...

Tasks as notes...
  - How can we attack this?
  - Do we store note data in notes?
  - What about searching?

For now lets just keep tasks as mostly in database but that are also linked to
notes.

So for now let's get a basic notes set up.

