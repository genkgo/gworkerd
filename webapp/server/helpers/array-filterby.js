if (!Array.prototype.filterBy) {
  Array.prototype.filterBy = function(search, value) {
    if (this == null) {
      throw new TypeError('Array.prototype.filterBy called on null or undefined');
    }

    var list = Object(this);
    var length = list.length >>> 0;
    var thisArg = arguments[1];
    var object;
    var result = [];
    for (var i = 0; i < length; i++) {
      object = list[i];
      var objectValue = object[search];
      if (objectValue === null && value == '') {
        result.push(object);
        continue;
      }
      if (object[search] && object[search] == value) {
        result.push(object);
      }
    }
    return result;
  };
}