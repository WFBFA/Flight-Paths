# Flight Paths Compute

_make 'em fly_

Using the road graph and surveillance vehicle configuration (in JSONs as per the schema), compute _somewhat_ optimal paths for perfect\* road coverage.

\* - unreachable road segments are unreachable and there's nothing i can do about it :P

The app is a Rust CLI - just run with `cargo bin`.

## Limitations

Current algorithm will not utilize all of the vehicles starting at the same graph node if there are more vehicles there than half the number of augmented edges at that node.

The vehicles are allowed to follow the road graph and only. That means that if there are _logically_ disconnected portions, even if they are physically accessible, they will not be visited (and you will get a warning).

Flight speed, traffic control, weather, fuel/🔋 mileage, and most other physical conditions are _not_ taken into account.

The lengths of paths of vehicles are balanced, to _some_ possible/reasonable extent.

## Example usage
1. get ur road graph in `montreal.roads.json`
2. create a vehicle configuration in `drones.json`. for example
```json
[
	"596644787",
	"218198673",
	"4234468198"
]
```
3. run `cargo bin -- fly montreal.roads.json drones.json drones.paths.json`
4. the paths for the 3 drones are now in `drones.paths.json`
5. shalt thou wish to geojsonify it, run `cargo bin -- geojson montreal.roads.json drones.paths.json drones.path` and make use of the generated `drones.path.1.geojson`, `drones.path.2.geojson` and `drones.path.3.geojson` files.
