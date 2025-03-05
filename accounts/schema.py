from graphene import (
    relay,
    ObjectType,
    Schema,
    Mutation,
    String,
    Field,
    Int,
    Boolean,
    List,
)
from graphene_django import DjangoObjectType
from graphene_django.filter import DjangoFilterConnectionField
from accounts.models import Account
from custom_user.models import CustomUser
from accounts.exceptions import AccountNameAlreadyExists, UserRequiredError


class AccountNode(DjangoObjectType):
    class Meta:
        model = Account
        filter_fields = "__all__"
        interfaces = (relay.Node,)

    @classmethod
    def __init_subclass_with_meta__(cls, **kwargs):
        from custom_user.schema import CustomUserNode

        cls._meta.fields["users"] = relay.ConnectionField(CustomUserNode)
        super().__init_subclass_with_meta__(**kwargs)


class CreateAccount(Mutation):
    class Arguments:
        name = String(required=True)
        user_ids = user_ids = List(Int, required=True)

    account = Field(AccountNode)

    def mutate(self, info, name, user_ids):
        if Account.objects.filter(name=name).exists():
            raise AccountNameAlreadyExists(
                f"An account with the name '{name}' already exists."
            )

        account = Account(name=name)
        account.save()

        users = CustomUser.objects.filter(id__in=user_ids)
        if not users:
            raise UserRequiredError("At least one user is required for the account.")

        account.users.set(users)
        account.save()

        return CreateAccount(account=account)


class UpdateAccount(Mutation):
    class Arguments:
        id = Int(required=True)
        name = String()

    account = Field(AccountNode)

    def mutate(self, info, id, name=None):
        try:
            account = Account.objects.get(id=id)
            if name:
                account.name = name
            account.save()
            return UpdateAccount(account=account)
        except Account.DoesNotExist:
            raise Exception(f"Account with id {id} does not exist.")


class DeleteAccount(Mutation):
    class Arguments:
        id = Int(required=True)

    success = Boolean()

    def mutate(self, info, id):
        try:
            account = Account.objects.get(id=id)
            account.delete()
            return DeleteAccount(success=True)
        except Account.DoesNotExist:
            return DeleteAccount(success=False)


class Query(ObjectType):
    account = relay.Node.Field(AccountNode)
    all_accounts = DjangoFilterConnectionField(AccountNode)


class Mutation(ObjectType):
    create_account = CreateAccount.Field()
    update_account = UpdateAccount.Field()
    delete_account = DeleteAccount.Field()


schema = Schema(query=Query, mutation=Mutation, types=[AccountNode])
