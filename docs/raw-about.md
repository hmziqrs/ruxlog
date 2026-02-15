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

admin & consumer are both dioxus projects but they're separate because they have different dependencies and build targets. consumer is built with SSR in mind while admin is purely SPA and has some heavy dependencies like code editors, image editors, & photon.rs that we don't want to bloat the consumer bundle. Although both projects are cross-platform and runs on mobile, desktop, and web. but some web specific features like code editor and image editor only enabled for web, because they heavily rely on browser API.

Architecture and coding standards on complete repo was hand written by me for initial modules. for example each module in sea_models has 4 files (will be 3 in future code revamp) actions.rs (abstraction for db related calls), mod.rs, model.rs (model schema in rust along with relations & keys), slice.rs (structs for create, update, & complete object with relations). But repeated stuff/boilerplate, migrations and some other filler stuff is AI generated.
To specific I use claude-code, codex, opencode (glm), kilo code & github copilot.
I've been inconsistently doing development on this project had to pause bunch of time mostly health and personal issues. So the codebase is mess of AI and hand written code with a few inconsistent coding standard.

After completing majority of admin frontend in dioxus I had second thoughts about using dioxus for consumer frontend. Just because of how time consuming it is do things here because of non existence eco-system. But initially I wanted to build everything in rust including frontend. So I stuck with dioxus for consumer as well no matter how much time consuming it is. and also I though this would hurt my pride.


Also one of my main blocker was AI! Thing is AI let's plan and implement new features so easily instead of improving it slowed me down by having to test and manage it. In my code base one can find so stuff like OTEL authentication, comments, user profile pages, reports, banning and whole let's of full flegded stuff. but implementing all of them via vibe coding slowed me down instead of providing value.

Also goal was to test full cross platform capablities of dioxus and also write some rust interpolation for core native app packages like firebase analytics, crashlytics and notifications and what not but I have scrapped the idea as I'm not very fluent in rust and vibe coding it would do more harm than good. So my goal thinned out to just release a very basic read only blog but at least provide all binaries for desktop and android.

Also another goal was to check if would make a viable option for as an starter kit for admin, app and marketing page. but lack of UI framework and building things from scratch and keep iterating and fixing stuff would be counteractive as an starter kit meanwhile ther are tons of options of great options available like builiding your own infra on a vps or using managed paas like vercel, neon, and cloudflare.

I still believe in rust and it's growing eco-system of projects like Axum, Dioxus, and Bevy. but currently at this state the eco-system is not just viable for fastpace development. Even in terms of performance using bun with typescript provides near to go performance minus the complexity of rust. I would like to re visit Dioxus again in next 2 years with the eco system is more mature and there are properly maintained libaries are available for general stuff like firebase anlaytics, and notifications. just my thought meanwhile we use mature projects like tanstack, astro, nextjs, react native, flutter and tauri.

Not to mention the documentation and out dated example for doing basic stuff like SSG/SSR was such a pain to configure again this a deal breaker for fast pace development cycles and a very big demotivator when your stuck in sucha menial error which other frameworks handles so elegantly. Fixing small thing by going deep in the core repository filtering out examples using similar patterns search manually and going the other end of the internet (google page-3) and find a dangling article or youtube video stuff like that doesn't screams fast pace. neither it's inevitable becuase it's part of development all of this will get better as the framework get mature.

So will I stop using rust and dioxus ? No, I will keep using rust as time to time I tend to solve hackerrank questions in rust? yes I will pause creating propjects with this. will wait for eco-system to get more mature at least "properly" UI libraries would be massive inidicator tbh. This was my 3rd dioxus project ngl I've had so much fun using it. not having to do mental gymnastics for using use_state in a server is so refreshing to me. and to have new hybrid approach for data fetching in server and client is just awesome.

And Dioxus is still being developed actively and I have very high hopes for Diopsus native render. I think this would be an absolutely amazing replacement for Tori and Electron apps because no matter what mental gymnastics web view or web-based desktop application would do, but nothing can beat the experience and the feel a native renderer provides. This is still one of the reason I still prefer flutter because it use GPU rendering for all mobile and desktop platforms. Yeah, flutter web is finicky because instead of relying on native DOM it renders everything on a canvas. This is why this is what the biggest pain point IOX is trying to solve, which I think will have huge potential when native renders are fixed and common use case libraries are available for example like I previously mentioned push notifications analytics like that
