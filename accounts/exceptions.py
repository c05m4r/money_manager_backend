class AccountError(Exception):
    pass


class AccountNameAlreadyExists(AccountError):
    def __init__(self, message="Account name already exists"):
        self.message = message
        super().__init__(self.message)


class UserRequiredError(AccountError):
    def __init__(self, message="At least one user must be assigned to the account"):
        self.message = message
        super().__init__(self.message)
