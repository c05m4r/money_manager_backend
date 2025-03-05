from graphene import (
    relay,
    ObjectType,
    Schema,
    Mutation,
    String,
    Field,
    Int,
    Boolean,
    ID,
)
from graphene_django import DjangoObjectType
from graphene_django.filter import DjangoFilterConnectionField
from custom_user.models import CustomUser


class CustomUserNode(DjangoObjectType):
    class Meta:
        model = CustomUser
        filter_fields = "__all__"
        interfaces = (relay.Node,)

    @classmethod
    def __init_subclass_with_meta__(cls, **kwargs):
        from accounts.schema import AccountNode

        cls._meta.fields["accounts"] = relay.ConnectionField(AccountNode)
        super().__init_subclass_with_meta__(**kwargs)

class CreateCustomUser(Mutation):
    class Arguments:
        username = String(required=True)
        email = String(required=True)
        password = String(required=True)

    user = Field(CustomUserNode)

    def mutate(self, info, username, email, password):
        if CustomUser.objects.filter(username=username).exists():
            raise Exception(f"A user with the username '{username}' already exists.")

        user = CustomUser(username=username, email=email)
        user.set_password(password)
        user.save()

        return CreateCustomUser(user=user)


class UpdateCustomUser(Mutation):
    class Arguments:
        # id = Int(required=True)
        id = ID(required=True)
        username = String()
        email = String()
        password = String()

    user = Field(CustomUserNode)

    def mutate(self, info, id, username=None, email=None, password=None):
        # print(f"Received id: {id}")  # Depuración del id
        try:
            user = CustomUser.objects.get(pk=id)
            # decoded_id = CustomUserNode.get_node_from_global_id(info, id, CustomUser)
            # decoded_id = CustomUserNode.get_node_from_global_id(id)[1]
            # print(f"Decoded id: {id}")  # Depuración del id
            # user = CustomUser.objects.get(id=decoded_id)
            # user = CustomUser.objects.filter(id=id)
            if username:
                user.username = username
            if email:
                user.email = email
            if password:
                user.set_password(password)
            user.save()
            return UpdateCustomUser(user=user)
        except CustomUser.DoesNotExist:
            raise Exception(f"User with id {id} does not exist.")


class DeleteCustomUser(Mutation):
    class Arguments:
        id = ID(required=True)

    success = Boolean()

    def mutate(self, info, id):
        try:
            user = CustomUser.objects.get(id=id)
            user.is_active = False
            user.save()
            return DeleteCustomUser(success=True)
        except CustomUser.DoesNotExist:
            return DeleteCustomUser(success=False)


class DeleteCustomUserByUsername(Mutation):
    class Arguments:
        username = String(required=True)

    success = Boolean()

    def mutate(self, info, username):
        try:
            user = CustomUser.objects.get(username=username)
            user.is_active = False
            user.save()
            return DeleteCustomUserByUsername(success=True)
        except CustomUser.DoesNotExist:
            return DeleteCustomUserByUsername(success=False)


class Query(ObjectType):
    user = relay.Node.Field(CustomUserNode)
    all_users = DjangoFilterConnectionField(CustomUserNode)


class Mutation(ObjectType):
    create_custom_user = CreateCustomUser.Field()
    update_custom_user = UpdateCustomUser.Field()
    delete_custom_user = DeleteCustomUser.Field()
    delete_custom_user_by_username = DeleteCustomUserByUsername.Field()


schema = Schema(query=Query, mutation=Mutation, types=[CustomUserNode])
