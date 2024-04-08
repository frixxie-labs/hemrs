POST http://localhost:65534/api/devices
```json
{
 "name": "test",
 "location": "test"
}
```
HTTP 200

GET http://localhost:65534/api/devices
HTTP 200

POST http://localhost:65534/
```json
{
"room": "test",
"temp": 20.2,
"hum": 50.0
}
```
HTTP 200

GET http://localhost:65534/api/measurements
HTTP 200
