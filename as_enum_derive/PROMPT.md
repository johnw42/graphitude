Generate a "derive" macro in this folder and a helper library based on the
example code in `directedness2.rs`.  Add the new crate(s) to the existing
workspace.

The macro should be named AsEnum.  It applies only to enums without data.  It
should generate a unit struct for each enum member.  The enum and the generated
structs should implement `AsEnum<T>`, where T is the enum.