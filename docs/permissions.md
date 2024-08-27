# Permissions
| Value     | Name                  | Description                                                                                       |
|-----------|-----------------------|---------------------------------------------------------------------------------------------------|
| `1 << 0`  | `READ_PUBLIC_THREADS` | Allows for reading non-locked threads                                                             |
| `1 << 1`  | `CREATE_THREADS`      | Allows creation of threads                                                                        |
| `1 << 2`  | `MANAGE_THREADS`      | Allows management and editing of threads                                                          |
| `1 << 3`  | `SEND_MESSAGES`       | Allows for sending messages in threads                                                            |
| `1 << 4`  | `MANAGE_MESSAGES`     | Allows for deletion of other users messages                                                       |
| `1 << 5`  | `ADD_REACTIONS`       | Allows for the addition of reactions to messages                                                  |
| `1 << 6`  | `MANAGE_CATEGORIES`   | Allows management, creation and editing of categories                                             |
| `1 << 7`  | `MANAGE_USERS`        | Allows for editing other user's usernames, display names                                          |
| `1 << 8`  | `MODERATE_USERS`      | Allows for timing out and banning users                                                           |
| `i64 MAX` | `ADMINISTRATOR`       | Allows all permissions and grants access to all endpoints (This is dangerous permission to grant) |