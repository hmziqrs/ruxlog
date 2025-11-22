Let's write our about page in consumer-dioxus

Let's first first write down our project structure:
A mono repo.
    - backend
      - api (axum)
        - tui (ratatui)
        - 
      - docker configs
    - frontend
      - consumer (dioxus)
      - admin (dioxus)
      - shared (project specific shared code)
      - oxform (form library)
      - oxui (ui component library)
      - oxstore (state management library)
      - oxcore (core utilities)


Now details:

admin & consumer are both dioxus projects but they're separate because they have different dependencies and build targets. consumer is built with SSR in mind while admin is purely SPA and has some heavy dependencies like code editors, image editors, & photon.rs that we don't want to bloat the consumer bundle with. Although both projects are cross-platform and runs on mobile, desktop, and web. but some web specific features like code editor and image editor only enabled for web, because they heavily rely on browser API. 

Although the architecture and coding standards on complete repo was hand written by me for initial modules. for example each module in sea_models has 4 files (will be 3 in future code revamp) actions.rs (abstraction for db related calls), mod.rs, model.rs (model schema in rust along with relations & keys), slice.rs (structs for create, update, & complete object with relations).
