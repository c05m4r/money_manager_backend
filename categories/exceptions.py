class CategoryError(Exception):
    pass


class CategoryNotFoundError(CategoryError):
    def __init__(self, message="Category not found"):
        self.message = message
        super().__init__(self.message)
