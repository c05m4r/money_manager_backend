from graphene import relay, ObjectType, Schema, Mutation, String, Field, Int, Boolean
from graphene_django import DjangoObjectType
from graphene_django.filter import DjangoFilterConnectionField
from categories.models import Category


class CategoryNode(DjangoObjectType):
    class Meta:
        model = Category
        filter_fields = "__all__"
        interfaces = (relay.Node,)


class CreateCategory(Mutation):
    class Arguments:
        name = String(required=True)

    category = Field(CategoryNode)

    def mutate(self, info, name):
        if Category.objects.filter(name=name).exists():
            raise Exception(f"A category with the name '{name}' already exists.")
        category = Category(name=name)
        category.save()
        return CreateCategory(category=category)


class UpdateCategory(Mutation):
    class Arguments:
        id = Int(required=True)
        name = String()

    category = Field(CategoryNode)

    def mutate(self, info, id, name=None):
        try:
            category = Category.objects.get(id=id)
            if name:
                if Category.objects.filter(name=name).exists():
                    raise Exception(
                        f"A category with the name '{name}' already exists."
                    )
                category.name = name
            category.save()
            return UpdateCategory(category=category)
        except Category.DoesNotExist:
            raise Exception(f"Category with id {id} does not exist.")


class DeleteCategory(Mutation):
    class Arguments:
        id = Int(required=True)

    success = Boolean()

    def mutate(self, info, id):
        try:
            category = Category.objects.get(id=id)
            category.delete()
            return DeleteCategory(success=True)
        except Category.DoesNotExist:
            return DeleteCategory(success=False)


class DeleteCategoryByName(Mutation):
    class Arguments:
        name = String(required=True)

    success = Boolean()

    def mutate(self, info, name):
        try:
            category = Category.objects.get(name=name)
            category.delete()
            return DeleteCategoryByName(success=True)
        except Category.DoesNotExist:
            return DeleteCategoryByName(success=False)


class Mutation(ObjectType):
    create_category = CreateCategory.Field()
    update_category = UpdateCategory.Field()
    delete_category = DeleteCategory.Field()
    delete_category_by_name = DeleteCategoryByName.Field()


class Query(ObjectType):
    category = relay.Node.Field(CategoryNode)
    all_categories = DjangoFilterConnectionField(CategoryNode)


schema = Schema(query=Query, mutation=Mutation, types=[CategoryNode])
