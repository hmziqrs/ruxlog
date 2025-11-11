# Edit State Abstraction

## Overview

The `edit_state_abstraction_with_list` function provides a reusable pattern for handling update/edit operations in the store layer. It manages the complete lifecycle of an edit request, including loading states, error handling, and **automatic cache synchronization** via flags.

## Function Signatures

### For PaginatedList Caches (Most Common)

```rust
pub async fn edit_state_abstraction_with_list<K, T, Payload, F, GetId, OnSuccess>(
    state: &GlobalSignal<HashMap<K, StateFrame<(), Payload>>>,
    id: K,
    payload: Payload,
    send_future: F,
    parse_label: &str,
    sync_list_cache: Option<&GlobalSignal<StateFrame<PaginatedList<T>>>>,
    sync_view_cache: Option<&GlobalSignal<HashMap<K, StateFrame<T>>>>,
    get_id: GetId,
    on_success: Option<OnSuccess>,
) -> Option<T>
where
    K: Eq + Hash + Copy + 'static,
    T: DeserializeOwned + Clone + PartialEq + 'static,
    Payload: Clone + 'static,
    F: Future<Output = Result<HttpResponse, HttpError>>,
    GetId: Fn(&T) -> K,
    OnSuccess: FnOnce(&T),
```

### For Generic List Caches

```rust
pub async fn edit_state_abstraction<K, ListData, Parsed, F, GetId, OnSuccess>(
    state: &GlobalSignal<HashMap<K, StateFrame<(), Parsed>>>,
    id: K,
    payload: Parsed,
    send_future: F,
    parse_label: &str,
    sync_list_cache: Option<&GlobalSignal<StateFrame<ListData>>>,
    sync_view_cache: Option<&GlobalSignal<HashMap<K, StateFrame<Parsed>>>>,
    get_id: GetId,
    on_success: Option<OnSuccess>,
) -> Option<Parsed>
```

## Parameters

- **state**: A `GlobalSignal` containing a `HashMap` keyed by ID, where each entry is a `StateFrame` tracking the edit operation
- **id**: The unique identifier of the resource being edited
- **payload**: The data payload being sent to the server (stored as metadata in the StateFrame)
- **send_future**: The HTTP request future (typically from `http_client::post(...).send()`)
- **parse_label**: A label used in error messages (e.g., "tag", "user", "category")
- **sync_list_cache**: Optional reference to the list cache to automatically update
- **sync_view_cache**: Optional reference to the view cache to automatically update
- **get_id**: Function to extract the ID from the parsed response (e.g., `|tag: &Tag| tag.id`)
- **on_success**: Optional callback for additional custom logic after successful edit

## Return Value

Returns `Option<Parsed>`:
- `Some(parsed_data)` if the request succeeded
- `None` if the request failed (network error, API error, or parse error)

## Usage Example

### Before (Manual Implementation)

```rust
pub async fn edit(&self, id: i32, payload: TagsEditPayload) {
    {
        let mut edit_map = self.edit.write();
        edit_map
            .entry(id)
            .or_insert_with(StateFrame::new)
            .set_loading_meta(Some(payload.clone()), None);
    }

    let result = http_client::post(&format!("/tag/v1/update/{}", id), &payload)
        .send()
        .await;
    match result {
        Ok(response) => {
            if (200..300).contains(&response.status()) {
                match response.json::<Tag>().await {
                    Ok(tag) => {
                        {
                            let mut edit_map = self.edit.write();
                            edit_map
                                .entry(id)
                                .or_insert_with(StateFrame::new)
                                .set_success(None, None);
                        }
                        
                        // Manually sync list cache
                        if let Some(list) = &self.list.peek().data {
                            if list.data.iter().any(|t| t.id == id) {
                                let mut list_frame = self.list.write();
                                if let Some(list) = &mut list_frame.data {
                                    if let Some(item_mut) = list.data.iter_mut().find(|t| t.id == id) {
                                        *item_mut = tag.clone();
                                    }
                                }
                            }
                        }
                        
                        // Manually sync view cache
                        let mut view_map = self.view.write();
                        if let Some(view_frame) = view_map.get_mut(&id) {
                            view_frame.set_success(Some(tag.clone()), None);
                        }
                    }
                    Err(e) => {
                        let mut edit_map = self.edit.write();
                        edit_map
                            .entry(id)
                            .or_insert_with(StateFrame::new)
                            .set_failed(Some(format!("Failed to parse tag: {}", e)));
                    }
                }
            } else {
                let mut edit_map = self.edit.write();
                edit_map
                    .entry(id)
                    .or_insert_with(StateFrame::new)
                    .set_api_error(&response)
                    .await;
            }
        }
        Err(e) => {
            let mut edit_map = self.edit.write();
            edit_map
                .entry(id)
                .or_insert_with(StateFrame::new)
                .set_failed(Some(format!("Network error: {}", e)));
        }
    }
}
```

### After (Using Abstraction)

```rust
pub async fn edit(&self, id: i32, payload: TagsEditPayload) {
    let _tag = edit_state_abstraction_with_list(
        &self.edit,
        id,
        payload.clone(),
        http_client::post(&format!("/tag/v1/update/{}", id), &payload).send(),
        "tag",
        Some(&self.list),        // Automatically syncs list cache
        Some(&self.view),        // Automatically syncs view cache
        |tag: &Tag| tag.id,      // ID extractor
        None::<fn(&Tag)>,        // No custom callback needed
    )
    .await;
}
```

## Benefits

1. **Minimal Boilerplate**: Reduced from ~65 lines to ~10 lines (85% reduction!)
2. **Automatic Cache Sync**: No manual cache validation or updates needed
3. **Consistency**: Ensures all edit operations follow the same pattern
4. **Type Safety**: Generic implementation with proper type constraints
5. **Flexible**: Optional callbacks for additional custom logic
6. **Maintainability**: Centralized error handling and state management logic

## State Lifecycle

1. **Loading**: Sets the state to loading with the payload as metadata
2. **Request**: Sends the HTTP request
3. **Success**: 
   - Parses the response
   - Sets state to success
   - **Automatically updates list cache** (if provided)
   - **Automatically updates view cache** (if provided)
   - Invokes `on_success` callback for additional custom logic (if provided)
   - Returns `Some(parsed_data)`
4. **Failure**:
   - Sets appropriate error message (network, API, or parse error)
   - Returns `None`

## Cache Synchronization Flags

### `sync_list_cache`

Pass `Some(&self.list)` to automatically update the list cache:
- Finds the item in the list by ID using the `get_id` function
- Replaces it with the updated item from the server
- No manual validation or iteration needed

```rust
Some(&self.list),  // Enable list sync
```

### `sync_view_cache`

Pass `Some(&self.view)` to automatically update the view cache:
- Updates the view cache entry for the given ID
- Sets it to success with the updated item
- Keeps detail views in sync

```rust
Some(&self.view),  // Enable view sync
```

### No Cache Sync

Pass `None` to skip cache synchronization:

```rust
None,  // No list sync
None,  // No view sync
```

## Custom Logic with `on_success`

If you need additional logic after a successful edit, provide a callback:

```rust
edit_state_abstraction_with_list(
    &self.edit,
    id,
    payload.clone(),
    request.send(),
    "tag",
    Some(&self.list),
    Some(&self.view),
    |tag: &Tag| tag.id,
    Some(|tag: &Tag| {
        // Custom logic: e.g., refresh related resources
        // spawn(async move { fetch_related_items(tag.category_id).await });
        log::info!("Tag {} updated successfully", tag.name);
    }),
)
.await;
```

## Comparison Table

| Feature | Manual Implementation | With Abstraction |
|---------|----------------------|------------------|
| Lines of Code | ~65 lines | ~10 lines |
| Error Handling | Manual match/if blocks | Automatic |
| Cache Sync | Manual validation & updates | Automatic via flags |
| Type Safety | Manual type annotations | Generic inference |
| Consistency | Varies per implementation | Enforced pattern |
| Maintenance | Update each action individually | Update once in lib.rs |

## Examples for Different Scenarios

### Sync Both Caches

```rust
edit_state_abstraction_with_list(
    &self.edit,
    id,
    payload.clone(),
    http_client::post(url, &payload).send(),
    "user",
    Some(&self.list),  // ✅ Sync list
    Some(&self.view),  // ✅ Sync view
    |user: &User| user.id,
    None::<fn(&User)>,
)
.await;
```

### Sync List Only

```rust
edit_state_abstraction_with_list(
    &self.edit,
    id,
    payload.clone(),
    http_client::post(url, &payload).send(),
    "category",
    Some(&self.list),  // ✅ Sync list
    None,              // ❌ No view sync
    |cat: &Category| cat.id,
    None::<fn(&Category)>,
)
.await;
```

### Sync View Only

```rust
edit_state_abstraction_with_list(
    &self.edit,
    id,
    payload.clone(),
    http_client::post(url, &payload).send(),
    "post",
    None,               // ❌ No list sync
    Some(&self.view),  // ✅ Sync view
    |post: &Post| post.id,
    None::<fn(&Post)>,
)
.await;
```

### No Cache Sync (Just Track Edit State)

```rust
edit_state_abstraction_with_list(
    &self.edit,
    id,
    payload.clone(),
    http_client::post(url, &payload).send(),
    "setting",
    None,  // ❌ No list sync
    None,  // ❌ No view sync
    |s: &Setting| s.id,
    None::<fn(&Setting)>,
)
.await;
```

## Related Abstractions

- `state_request_abstraction`: For create/add operations with a single StateFrame
- `view_state_abstraction`: For fetching a single record by ID
- `list_state_abstraction`: For fetching paginated lists

## Implementation Location

- Function: `/src/store/lib.rs` (lines 310-474)
- Example usage: `/src/store/tags/actions.rs` (lines 27-38)
