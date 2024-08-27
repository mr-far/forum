# Forum

A small practical REST API forum with WebSocket.

### How to use

1. Rename `.env.example` to `.env`, after fill all fields
2. Migrate database schema with `sqlx migrate`
3. Build and start project with `cargo` commands

### TODO

- [ ] Gateway
  - [x] Send and Receive events
  - [ ] Add user to online list after identification
- [ ] Authorization
    - [x] Sessions
      - [x] Sessions on tokens
      - [ ] Manage sessions like in Discord (View, Delete, etc.)
    - [ ] Multi-factor authentication (MFA)
    - [ ] Forget password function
- [ ] Messages
  - [x] Send, Delete, Edit
  - [x] Replies
  - [ ] Reactions
  - [ ] Attachments
  - [ ] Reports 
- [ ] Threads
  - [ ] More types (Private, Pass Requirements To Join)
  - [ ] Flags modification (NSFW, Locked, Pinned)
- [ ] Moderation
  - [ ] Bans, timeouts
  - [ ] Reports management
- [ ] Users
  - [x] Permissions and flags
  - [ ] Customizable profiles (Banners, Colorful display names)
- [x] Documentation