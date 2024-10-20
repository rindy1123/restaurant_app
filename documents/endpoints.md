# Endpoints

This kind of document should eventually be generated automatically.

- `GET /tables/:table_id/order_items`
    - Response
```json
{
    "items": [
        {
            "id": 1,
            "table_number": 1,
            "menu_item_name": "Big Mac",
            "prep_time_minutes": 10
        },
        {
            "id": 2,
            "table_number": 2,
            "menu_item_name": "French Fries",
            "prep_time_minutes": 15
        }
    ]
}
```
- `GET /tables/:table_id/order_items/:order_item_id`
    - Response
```json
{
    "id": 1,
    "table_number": 1,
    "menu_item_name": "Big Mac",
    "prep_time_minutes": 10
}
```
- `POST /tables/:table_id/orders`
    - Request Body
```json
{
    "menu_item_ids": [1, 2, 3]
}
```
- `DELETE /tables/:table_id/order_items/:order_item_id`
