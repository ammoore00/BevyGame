# Reference Syntax Guide
- `<arg>` = Required Argument
- `[arg]` = Optional Argument
- `key=val` = Key-Value Pair Argument
- `...` = Repeatable Argument
- `|` = Mutually Exclusive Choices (e.g. `(on|off)`)

# Commands
## Spawn
**Syntax:**
`spawn <resource> [coords]`

**Arguments:**
- `<resource>` = Resource location of the entity to spawn
- `[coords]` = Coordinates to spawn the entity at

## Character
**Syntax:**
`character <entity> <operation>`

**Arguments:**
- `<entity>` = Entity to perform the operation on
- `<operation>` = Operation to perform on the entity

### Operations
**Syntax:**
- `modify <attribute> <value>`

**Arguments:**
- `<attribute>` = Attribute to modify
  - `health`: unsigned int
- `<value>` = Value to set the attribute to

## Example
**Syntax:**


**Arguments:**
